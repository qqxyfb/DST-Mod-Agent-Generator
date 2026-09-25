//! DST 游戏目录校验与原版脚本白名单读取（PROJECT_SPEC.md §4.1.1 / §7.8）
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct DstInfo {
    pub valid: bool,
    pub errors: Vec<String>,
    pub scripts_count: usize,
    pub sample_scripts: Vec<String>,
}

/// 校验 DST 根目录：必须存在 mods/ 与 scripts/
pub fn validate(root: &str) -> DstInfo {
    let path = Path::new(root);
    if !path.is_dir() {
        return DstInfo {
            valid: false,
            errors: vec![format!("目录不存在: {root}")],
            scripts_count: 0,
            sample_scripts: vec![],
        };
    }
    let mut errors = vec![];
    for sub in ["mods", "scripts"] {
        if !path.join(sub).is_dir() {
            errors.push(format!("缺少 {sub}/ 文件夹"));
        }
    }
    let scripts = match read_whitelist(root) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            vec![]
        }
    };
    let scripts_count = scripts.len();
    let sample_scripts = scripts
        .iter()
        .take(5)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    DstInfo {
        valid: errors.is_empty(),
        errors,
        scripts_count,
        sample_scripts,
    }
}

/// 白名单：关键人物 prefab + 常用组件（控制参考上下文 token 预算）
const WHITELIST: [&str; 9] = [
    "prefabs/wilson.lua",
    "prefabs/wolfgang.lua",
    "prefabs/wickerbottom.lua",
    "prefabs/wes.lua",
    "components/health.lua",
    "components/hunger.lua",
    "components/sanity.lua",
    "components/inventory.lua",
    "components/combat.lua",
];

pub fn read_whitelist(root: &str) -> Result<Vec<PathBuf>, String> {
    let base = Path::new(root).join("scripts");
    if !base.is_dir() {
        return Err(format!("scripts/ 目录不存在: {}", base.display()));
    }
    let mut out = vec![];
    for rel in WHITELIST {
        let full = base.join(rel);
        if full.is_file() {
            out.push(full);
        }
    }
    Ok(out)
}

/// 拼接白名单脚本内容（受限长度），供 LLM 参考上下文注入
pub fn whitelist_snippets(root: &str, max_chars: usize, per_file: usize) -> Result<String, String> {
    let files = read_whitelist(root)?;
    let mut buf = String::new();
    for f in files {
        if buf.len() >= max_chars {
            break;
        }
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.lua")
            .to_string();
        if let Ok(content) = std::fs::read_to_string(&f) {
            let cut = content.chars().take(per_file).collect::<String>();
            buf.push_str(&format!("\n--- {name} ---\n{cut}\n"));
        }
    }
    Ok(buf)
}

pub fn scripts_index_json(root: &str) -> Result<Vec<String>, String> {
    let files = read_whitelist(root)?;
    Ok(files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}