//! Tab4 测试与调试相关 command
//! 读取 DST 日志、LLM 修复 Lua、生成控制台调试命令
use crate::core::{config, console, llm, logreader, project};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct FixResult {
    pub applied: Vec<String>,
    pub ok: bool,
    pub message: String,
}

/// 自动定位 DST 日志（Documents\Klei 递归探测）
#[tauri::command]
pub fn locate_logs() -> Result<Vec<logreader::LogFile>, String> {
    Ok(logreader::locate())
}

/// 读取日志（错误行高亮；超大日志截断到尾部 3000 行）
#[tauri::command]
pub fn read_log(path: String) -> Result<logreader::LogContent, String> {
    let mut c = logreader::read(&path)?;
    const CAP: usize = 3000;
    if c.lines.len() > CAP {
        let skip = c.lines.len() - CAP;
        c.lines.drain(..skip);
        let mut new_idx = vec![];
        for i in c.error_indices {
            if i >= skip {
                new_idx.push(i - skip);
            }
        }
        c.error_indices = new_idx;
    }
    Ok(c)
}

/// 一键修复：日志报错块 + 当前 mod 源码 → LLM 输出修复 JSON → 只改写受影响的 Lua 文件
#[tauri::command]
pub async fn fix_from_log(
    project_path: String,
    log_path: String,
) -> Result<FixResult, String> {
    let info = match project::load(&project_path) {
        Ok(i) => i,
        Err(e) => {
            return Ok(FixResult {
                applied: vec![],
                ok: false,
                message: e,
            })
        }
    };
    let mod_dir = Path::new(&info.path).join("mod");
    let mut files: Vec<(String, String)> = vec![];
    collect_mod_lua(&mod_dir, &mod_dir, &mut files);
    if files.is_empty() {
        return Ok(FixResult {
            applied: vec![],
            ok: false,
            message: "mod/ 目录下没有 Lua 文件，请先在 Tab3 执行 Stage5 代码生成。".into(),
        });
    }
    let tail = match logreader::tail(&log_path, 150) {
        Ok(t) => t,
        Err(e) => {
            return Ok(FixResult {
                applied: vec![],
                ok: false,
                message: e,
            })
        }
    };
    let cfg = config::load();
    if cfg.llm.base_url.trim().is_empty() || cfg.llm.model.trim().is_empty() {
        return Ok(FixResult {
            applied: vec![],
            ok: false,
            message: "请先在 Tab1 配置 LLM API（Base URL / 模型名称）。".into(),
        });
    }

    let mut body = format!("【DST 游戏日志（含报错，尾部 150 行）】\n{tail}\n\n【当前 mod Lua 源码】\n");
    for (rel, content) in &files {
        body.push_str(&format!("--- {rel} ---\n{content}\n"));
    }
    body.push_str(
        "\n请定位报错原因，仅修复有问题的文件。输出 JSON：{\"files\":[{\"path\":\"mod 内相对路径\",\"content\":\"修复后的完整文件内容\"}],\"note\":\"修复说明（中文）\"}。不要改动无关文件，不要返回 Markdown。",
    );
    let system = "你是《饥荒：联机版》(Don't Starve Together) 的 Lua 调试修复专家。严格遵循 DST Mod 规范：服务器逻辑置于 ismastersim 守卫内、客户端访问 ThePlayer 前判空、不污染全局作用域、只使用 DST 已知 API。只输出上述 JSON。";
    let client = llm::LlmClient::new(cfg.llm.clone());
    let reply = match client
        .chat(vec![
            llm::ChatMsg {
                role: "system".into(),
                content: system.into(),
            },
            llm::ChatMsg {
                role: "user".into(),
                content: body,
            },
        ])
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Ok(FixResult {
                applied: vec![],
                ok: false,
                message: format!("LLM 调用失败：{e}"),
            })
        }
    };
    let v = match llm::extract_json(&reply) {
        Some(v) => v,
        None => {
            return Ok(FixResult {
                applied: vec![],
                ok: false,
                message: "LLM 未返回可解析的修复 JSON，请重试或人工排查。".into(),
            })
        }
    };

    // 应用修复：只允许写 mod/ 目录内的 .lua 文件（防越界）
    let mut applied = vec![];
    if let Some(arr) = v.get("files").and_then(|f| f.as_array()) {
        for item in arr {
            let Some(rel) = item.get("path").and_then(|p| p.as_str()) else {
                continue;
            };
            let Some(content) = item.get("content").and_then(|c| c.as_str()) else {
                continue;
            };
            let rel = rel
                .trim()
                .trim_start_matches(|c| c == '/' || c == '\\');
            if rel.is_empty()
                || rel.contains("..")
                || !rel.ends_with(".lua")
            {
                continue;
            }
            let target = mod_dir.join(rel);
            if !target.starts_with(&mod_dir) {
                continue;
            }
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if std::fs::write(&target, content).is_ok() {
                applied.push(rel.to_string());
            }
        }
    }
    if applied.is_empty() {
        return Ok(FixResult {
            applied: vec![],
            ok: false,
            message: "修复结果未产生有效文件变更（请检查 LLM 返回的 path 是否为 mod 内相对路径）。".into(),
        });
    }
    Ok(FixResult {
        applied,
        ok: true,
        message: "已应用修复，请重启游戏测试。若仍报错可在 Tab4 再次执行一键修复（最多 3 轮）。".into(),
    })
}

/// 递归收集 mod 目录下全部 lua 文件（返回相对路径）
fn collect_mod_lua(dir: &Path, base: &Path, out: &mut Vec<(String, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_mod_lua(&p, base, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("lua") {
            let rel = p
                .strip_prefix(base)
                .map(|r| r.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if let Ok(content) = std::fs::read_to_string(&p) {
                out.push((rel, content));
            }
        }
    }
}

/// 生成当前角色的控制台调试命令
#[tauri::command]
pub fn gen_console_cmds(project_path: String) -> Result<Vec<String>, String> {
    let info = project::load(&project_path)?;
    Ok(console::gen_cmds(&info))
}
