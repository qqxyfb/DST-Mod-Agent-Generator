//! 全局配置（PROJECT_SPEC.md §4.1）：本地 JSON 存储，Key 不上传
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateConfig {
    pub enabled: bool,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub dst_dir: String,
    pub modtools_path: String,
    pub llm: LlmConfig,
    pub image: ImageConfig,
    pub update: UpdateConfig,
}

/// 程序当前版本（与 tauri.conf.json / package.json 保持一致）
pub const CURRENT_VERSION: &str = "0.1.0";

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dst-mod-agent-generator")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

/// 仓库根目录（开发环境）：由编译期 CARGO_MANIFEST_DIR（= src-tauri）推导
pub fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest)
}

/// 用户 Mod 项目输出根目录：
/// 开发环境（存在仓库内 src/ 或 projects/）优先仓库内 projects/，否则退回配置目录
pub fn projects_root() -> PathBuf {
    let repo = repo_root();
    let repo_projects = repo.join("projects");
    if repo_projects.is_dir() || repo.join("src").is_dir() {
        let _ = std::fs::create_dir_all(&repo_projects);
        return repo_projects;
    }
    let p = config_dir().join("projects");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn load() -> AppConfig {
    match std::fs::read_to_string(config_file()) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save(cfg: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    let s = serde_json::to_string_pretty(cfg).map_err(|e| format!("序列化配置失败: {e}"))?;
    std::fs::write(config_file(), s).map_err(|e| format!("写入配置失败: {e}"))
}

/// 截断长文本（提示/日志展示用）
pub fn truncate(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        let t: String = chars[..n].iter().collect();
        format!("{t}…")
    }
}
