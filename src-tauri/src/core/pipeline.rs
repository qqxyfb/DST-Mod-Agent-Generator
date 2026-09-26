//! 阶段状态机：阶段卡片 + 视觉线/代码线双管线（PROJECT_SPEC.md §5）
//! 每个阶段可独立执行/重跑/跳过；上游产物变更只标记下游"需重跑"，不自动级联
use crate::core::{codegen, project, schema};
use serde_json::{json, Value};
use std::path::Path;
use tauri::{AppHandle, Emitter};

/// 阶段定义（与前端 types/index.ts STAGES 一致）
pub const STAGES: [(&str, &str, &str); 7] = [
    ("stage1", "人设定稿校验", "shared"),
    ("stage2", "部件提示词", "visual"),
    ("stage3", "部件图像生成", "visual"),
    ("stage4", "图像后处理", "visual"),
    ("stage5", "代码生成", "code"),
    ("stage6", "资源编译", "code"),
    ("stage7", "输出 Mod 报告", "code"),
];

pub fn emit_progress(app: &AppHandle, project_name: &str, stage: &str, message: &str, percent: f64) {
    let _ = app.emit(
        "pipeline://progress",
        json!({
            "project": project_name,
            "stage": stage,
            "message": message,
            "percent": percent,
        }),
    );
}

/// 运行阶段（幂等：重跑 = 再次调用）
/// `hint` = 用户在 Tab3 填写的「重跑提示词」，为空则沿用该阶段上次保存的提示词；
/// 仅 LLM 驱动的阶段（当前为 stage5 代码生成）消费，其余阶段记录到状态与日志中备查。
pub async fn run_stage(
    app: AppHandle,
    project_path: &str,
    stage_id: &str,
    hint: Option<String>,
) -> Result<Value, String> {
    // 用户重跑提示词：非空则覆盖保存并写日志；为空则复用该阶段上次保存的提示词（点「重跑」即可复用）
    let mut hint = hint.unwrap_or_default().trim().to_string();
    if !hint.is_empty() {
        let mut info = project::load(project_path)?;
        let st = project::stage(&mut info, stage_id);
        st.hint = hint.clone();
        st.updated_at = chrono::Utc::now().to_rfc3339();
        st.log.push(format!("✎ 重跑提示词：{hint}"));
        project::save(&info)?;
    } else {
        hint = project::load(project_path)?
            .pipeline
            .get(stage_id)
            .map(|s| s.hint.clone())
            .unwrap_or_default();
    }
    run_stage_inner(app, project_path, stage_id, hint).await
}

/// 阶段执行主体：先把提示词写回状态后再分派
async fn run_stage_inner(
    app: AppHandle,
    project_path: &str,
    stage_id: &str,
    hint: String,
) -> Result<Value, String> {
    // 限制固化（PROJECT_SPEC.md §12）：图像线/编译线属 V1 预留，标记"已跳过"并记录原因
    if matches!(stage_id, "stage2" | "stage3" | "stage4" | "stage6") {
        let mut info = project::load(project_path)?;
        let st = project::stage(&mut info, stage_id);
        st.status = "skipped".into();
        st.updated_at = chrono::Utc::now().to_rfc3339();
        if !hint.is_empty() {
            st.log.push(format!("已记录重跑提示词（V1 接入后生效）：{hint}"));
        }
        st.log
            .push("V1 未接入（预留接口）：请在 V1 启用 rembg/SAM2/autocompiler 后再执行。".into());
        project::save(&info)?;
        return Ok(json!(&info.pipeline));
    }
    mark(project_path, stage_id, "running", "开始执行")?;
    let outcome = match stage_id {
        "stage1" => stage1(&app, project_path).await,
        "stage5" => stage5(&app, project_path, &hint).await,
        "stage7" => stage7(&app, project_path).await,
        other => Err(format!("未知阶段: {other}")),
    };
    let mut info = project::load(project_path)?;
    let st = project::stage(&mut info, stage_id);
    match &outcome {
        Ok(_) => {
            st.status = "success".into();
            st.log.push("✔ 阶段成功".into());
        }
        Err(e) => {
            st.status = "failed".into();
            st.log.push(format!("✘ 失败: {e}"));
        }
    }
    st.updated_at = chrono::Utc::now().to_rfc3339();
    project::save(&info)?;
    Ok(json!(&info.pipeline))
}

/// Stage1：人设定稿校验（有草稿则冻结；有定稿则复核）
async fn stage1(app: &AppHandle, project_path: &str) -> Result<(), String> {
    emit_progress(app, project_path, "stage1", "校验人设 JSON", 30.0);
    let mut info = project::load(project_path)?;
    let Some(ch) = info.character.clone() else {
        return Err("尚未产出人设，请先在 Tab2 完成人设对话".into());
    };
    if !ch.frozen {
        emit_progress(app, project_path, "stage1", "检测到草稿，执行校验并冻结", 60.0);
        let ver = project::freeze_character(&mut info)?;
        emit_progress(app, project_path, "stage1", &format!("已冻结人设版本 {ver}"), 100.0);
    } else {
        let sheet: schema::CharacterSheet =
            serde_json::from_value(ch.json).map_err(|e| format!("人设 JSON 解析失败: {e}"))?;
        let errs = sheet.validate();
        if !errs.is_empty() {
            return Err(errs.join("；"));
        }
        emit_progress(app, project_path, "stage1", "定稿复核通过", 100.0);
    }
    Ok(())
}

/// Stage5：代码生成（S3）
/// hint = 用户重跑提示词，会追加进 LLM 提示词，用于按预期方向调整生成结果
async fn stage5(app: &AppHandle, project_path: &str, hint: &str) -> Result<(), String> {
    emit_progress(app, project_path, "stage5", "开始代码生成", 5.0);
    if !hint.trim().is_empty() {
        emit_progress(app, project_path, "stage5", &format!("应用重跑提示词：{hint}"), 8.0);
    }
    let report = codegen::generate(app.clone(), project_path, hint).await?;
    if !report.errors.is_empty() {
        return Err(format!("必生成文件断言失败: {}", report.errors.join("；")));
    }
    emit_progress(app, project_path, "stage5", "代码生成完成", 100.0);
    Ok(())
}

/// Stage7：输出 Mod 报告（列出文件路径 + 测试指引）
async fn stage7(app: &AppHandle, project_path: &str) -> Result<(), String> {
    project::load(project_path)?;
    let mod_dir = Path::new(project_path).join("mod");
    let mut files = vec![];
    if mod_dir.is_dir() {
        collect_lua_files(&mod_dir, &mut files)?;
    }
    emit_progress(app, project_path, "stage7", "生成 Mod 报告", 100.0);
    let report = json!({
        "mod_dir": mod_dir.to_string_lossy(),
        "files": files,
        "notes": [
            "1. 将 mod 文件夹复制到 DST mods/ 目录（或使用资源编译的导出缓存）。",
            "2. 启动游戏，在 Mods 列表中启用本 Mod。",
            "3. 若出现 Lua 报错，前往 Tab4 读取日志并一键修复。",
            "4. 专属道具/技能需在游戏中实测验证。",
        ],
    });
    let mut info = project::load(project_path)?;
    info.report = Some(report);
    project::save(&info)?;
    Ok(())
}

fn collect_lua_files(dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("读取 mod 目录失败: {e}"))?;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_lua_files(&p, out)?;
        } else if p.extension().and_then(|x| x.to_str()) == Some("lua") {
            out.push(p.to_string_lossy().to_string());
        }
    }
    Ok(())
}

/// 标记阶段状态（running/failed/skipped 等）
fn mark(project_path: &str, stage_id: &str, status: &str, note: &str) -> Result<(), String> {
    let mut info = project::load(project_path)?;
    let st = project::stage(&mut info, stage_id);
    st.status = status.into();
    st.updated_at = chrono::Utc::now().to_rfc3339();
    st.log.push(note.into());
    project::save(&info)
}

/// 跳过阶段
pub fn skip(project_path: &str, stage_id: &str) -> Result<(), String> {
    mark(project_path, stage_id, "skipped", "用户手动跳过")
}

/// 阶段状态总览（pipeline map）
pub fn state(project_path: &str) -> Result<Value, String> {
    let info = project::load(project_path)?;
    Ok(serde_json::to_value(&info.pipeline).map_err(|e| e.to_string())?)
}

/// 某阶段日志文本
pub fn stage_log(project_path: &str, stage_id: &str) -> Result<String, String> {
    let info = project::load(project_path)?;
    Ok(info
        .pipeline
        .get(stage_id)
        .map(|s| s.log.join("\n"))
        .unwrap_or_default())
}

/// 生成/读取 Mod 报告
pub fn report(project_path: &str) -> Result<Value, String> {
    let mut info = project::load(project_path)?;
    if let Some(r) = &info.report {
        return Ok(r.clone());
    }
    let mod_dir = Path::new(project_path).join("mod");
    let mut files = vec![];
    if mod_dir.is_dir() {
        collect_lua_files(&mod_dir, &mut files)?;
    }
    let r = json!({
        "mod_dir": mod_dir.to_string_lossy(),
        "files": files,
        "notes": ["报告未生成，请先执行 Stage7。"],
    });
    info.report = Some(r.clone());
    project::save(&info)?;
    Ok(r)
}
