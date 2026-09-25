//! 代码生成 Skill（S3 / Stage5）：确定性模板 + LLM 内容填充
//! 输入：人设 JSON vN + 原版脚本参考；输出：全套 mod Lua 源码 + 必生成项断言
use crate::core::schema::CharacterSheet;
use crate::core::{config, dst, llm, project};
use serde::Serialize;
use std::path::Path;
use tauri::AppHandle;

use crate::core::pipeline::emit_progress;

#[derive(Debug, Clone, Serialize)]
pub struct CodegenFile {
    pub path: String,
    pub ok: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CodegenReport {
    pub files: Vec<CodegenFile>,
    pub errors: Vec<String>,
}

/// 主入口：生成 mod 源码到 <project>/mod/
pub async fn generate(app: AppHandle, project_path: &str) -> Result<CodegenReport, String> {
    let cfg = config::load();
    let info = project::load(project_path)?;
    let char = info
        .character
        .as_ref()
        .ok_or("尚未产出人设，请先完成 Tab2 对话定稿")?;
    if !char.frozen {
        return Err("人设尚未定稿，请先在 Stage1（或 Tab2）确认定稿".into());
    }
    let sheet: CharacterSheet =
        serde_json::from_value(char.json.clone()).map_err(|e| format!("人设 JSON 解析失败: {e}"))?;

    let dst_info = dst::validate(&cfg.dst_dir);
    if !dst_info.valid {
        return Err(format!("DST 目录未通过校验：{}", dst_info.errors.join("；")));
    }

    emit_progress(&app, project_path, "stage5", "读取原版参考脚本", 10.0);
    let reference = dst::whitelist_snippets(&cfg.dst_dir, 60_000, 8_000)?;

    if cfg.llm.base_url.is_empty() {
        return Err("请先在 Tab1 配置 LLM API".into());
    }
    let client = llm::LlmClient::new(cfg.llm.clone());
    let mod_dir = Path::new(project_path).join("mod");
    std::fs::create_dir_all(mod_dir.join("prefabs")).map_err(|e| format!("创建 mod 目录失败: {e}"))?;
    let mut report = CodegenReport::default();

    // 1) 确定性模板：modinfo.lua
    emit_progress(&app, project_path, "stage5", "生成 modinfo.lua", 20.0);
    write_file(&mod_dir, "modinfo.lua", &render_modinfo(&sheet, &info.meta), &mut report);

    // 2) 确定性骨架：modmain.lua
    emit_progress(&app, project_path, "stage5", "生成 modmain.lua", 35.0);
    write_file(&mod_dir, "modmain.lua", &render_modmain(&sheet), &mut report);

    // 3) LLM：人物 prefab
    emit_progress(&app, project_path, "stage5", "生成人物 prefab", 55.0);
    let sys = system_prompt(&reference);
    let prefab_user = format!(
        "角色人设 JSON：\n{}\n\n请生成 prefab/{}.lua 的完整源码（只输出 Lua 代码）。",
        serde_json::to_string_pretty(&char.json).map_err(|e| e.to_string())?,
        sheet.char_name
    );
    let prefab_code = client
        .chat(vec![
            llm::ChatMsg { role: "system".into(), content: sys.clone() },
            llm::ChatMsg { role: "user".into(), content: prefab_user },
        ])
        .await?;
    write_file(
        &mod_dir.join("prefabs"),
        &format!("{}.lua", sheet.char_name),
        &strip_code_fence(&prefab_code),
        &mut report,
    );

    // 4) LLM：台词表
    emit_progress(&app, project_path, "stage5", "生成台词表 speech", 75.0);
    let speech_user = format!(
        "角色人设 JSON：\n{}\n\n请生成 speech_{}.lua 的完整源码，返回 `return {{ ... }}` 台词表，键为 ANNOUNCE_/DESCRIBE_/ACTIONFAIL_ 子集（只输出 Lua 代码）。",
        serde_json::to_string_pretty(&char.json).map_err(|e| e.to_string())?,
        sheet.char_name
    );
    let speech_code = client
        .chat(vec![
            llm::ChatMsg { role: "system".into(), content: sys },
            llm::ChatMsg { role: "user".into(), content: speech_user },
        ])
        .await?;
    write_file(
        &mod_dir,
        &format!("speech_{}.lua", sheet.char_name),
        &strip_code_fence(&speech_code),
        &mut report,
    );

    // 5) LLM：专属道具 prefab
    if !sheet.special_items.is_empty() {
        emit_progress(&app, project_path, "stage5", "生成专属道具 prefab", 88.0);
        for item in &sheet.special_items {
            let item_user = format!(
                "请为专属道具生成 prefab/{}.lua（类型 {}，属性 {}），只输出 Lua 代码。",
                item.id,
                item.kind,
                serde_json::to_string(&item.stats).unwrap_or_default()
            );
            let code = client
                .chat(vec![
                    llm::ChatMsg { role: "system".into(), content: system_prompt(&reference) },
                    llm::ChatMsg { role: "user".into(), content: item_user },
                ])
                .await?;
            write_file(&mod_dir.join("prefabs"), &format!("{}.lua", item.id), &strip_code_fence(&code), &mut report);
        }
    }

    emit_progress(&app, project_path, "stage5", "输出校验", 95.0);
    // 必生成项断言
    let required = [
        "modinfo.lua".to_string(),
        "modmain.lua".to_string(),
        format!("prefabs/{}.lua", sheet.char_name),
        format!("speech_{}.lua", sheet.char_name),
    ];
    for f in required {
        let p = mod_dir.join(&f);
        if !p.is_file() {
            report.errors.push(format!("必生成文件缺失: {f}"));
        }
    }
    emit_progress(&app, project_path, "stage5", "完成", 100.0);
    Ok(report)
}

fn write_file(dir: &Path, rel: &str, content: &str, report: &mut CodegenReport) {
    let path = dir.join(rel);
    match std::fs::write(&path, content) {
        Ok(()) => {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            report.files.push(CodegenFile {
                path: path.to_string_lossy().into_owned(),
                ok: true,
                size,
            });
        }
        Err(e) => report.errors.push(format!("写入 {rel} 失败: {e}")),
    }
}

/// 去除 ```lua 围栏（若有）
fn strip_code_fence(s: &str) -> String {
    let mut t = s.trim().to_string();
    if t.starts_with("```") {
        let first_nl = t.find('\n').unwrap_or(t.len());
        t = t[first_nl..].trim_start().to_string();
        if t.ends_with("```") {
            let cut = t.len() - 3;
            t = t[..cut].trim_end().to_string();
        }
    }
    t
}

fn system_prompt(reference: &str) -> String {
    format!(
        "你是《饥荒：联机版》(Don't Starve Together) 的高级 Mod 开发者。严格遵循 DST 官方 Mod 规范输出 Lua 代码：\n\
         1) 服务器逻辑必须放在 `if not TheWorld.ismastersim then return end` 之后；\n\
         2) 客户端逻辑访问 ThePlayer 前判空；不污染全局作用域；\n\
         3) 组件/方法名只使用 DST 已知 API；三围按 stats 字段填写；\n\
         4) 只输出 Lua 代码本身，不要用 Markdown 代码围栏包裹，不要解释。\n\
         参考原版脚本（白名单截取，仅供参考）：\n{}",
        reference
    )
}

fn render_modinfo(sheet: &CharacterSheet, meta: &project::ModMeta) -> String {
    format!(
        "-- modinfo.lua（DST Mod Agent Generator 自动生成，请勿手动编辑）\n\
         return {{\n\
         \tname = \"{name}\",\n\
         \tdescription = \"{desc}\",\n\
         \tauthor = \"{author}\",\n\
         \tversion = \"{version}\",\n\
         \tapi_version = 10,\n\
         \tdst_compatible = true,\n\
         \tdont_starve_compatible = false,\n\
         \tall_clients_require_mod = true,\n\
         \tconfiguration_options = {{}}\n\
         }}\n",
        name = sheet.char_name,
        desc = meta.description.replace('"', "'"),
        author = meta.author.replace('"', "'"),
        version = meta.version,
    )
}

fn render_modmain(sheet: &CharacterSheet) -> String {
    let items: Vec<String> = sheet.special_items.iter().map(|i| format!("\"{}\"", i.id)).collect();
    let items_lua = if items.is_empty() {
        String::new()
    } else {
        format!(", {}", items.join(", "))
    };
    format!(
        "-- modmain.lua（DST Mod Agent Generator 自动生成骨架）\n\
         local CHARNAME = \"{cname}\"\n\
         local CHARNAME_UP = string.upper(CHARNAME)\n\
         \n\
         -- 1) 注册 prefab（人物 + 专属道具）\n\
         PrefabFiles = {{ \"{cname}\"{items} }}\n\
         \n\
         -- 2) 台词与本地化\n\
         GLOBAL.STRINGS.CHARACTERS[CHARNAME_UP] = require(\"speech_{cname}\")\n\
         \n\
         -- 3) 注册角色（性别参数可按模板惯例调整）\n\
         -- 服务器侧逻辑统一放在对应 prefab 的 ismastersim 守卫内\n\
         AddModCharacter(CHARNAME, \"NEUTRAL\")\n",
        cname = sheet.char_name,
        items = items_lua,
    )
}