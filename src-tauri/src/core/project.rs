//! Mod 项目模型与 project.json 存储（PROJECT_SPEC.md §6 / §7.4）
use crate::core::config;
use crate::core::schema::CharacterSheet;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModMeta {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArtifact {
    pub version: String,
    pub json: Value,
    #[serde(default)]
    pub markdown: String,
    #[serde(default)]
    pub frozen: bool,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageState {
    pub status: String,
    #[serde(default)]
    pub artifact_version: Option<String>,
    #[serde(default)]
    pub input_hash: Option<String>,
    #[serde(default)]
    pub log: Vec<String>,
    #[serde(default)]
    pub updated_at: String,
}

impl Default for StageState {
    fn default() -> Self {
        Self {
            status: "pending".into(),
            artifact_version: None,
            input_hash: None,
            log: vec![],
            updated_at: now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub meta: ModMeta,
    pub created_at: String,
    #[serde(default)]
    pub character: Option<CharacterArtifact>,
    #[serde(default)]
    pub pipeline: HashMap<String, StageState>,
    #[serde(default)]
    pub report: Option<Value>,
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn project_file(dir: &Path) -> std::path::PathBuf {
    dir.join("project.json")
}

pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn create(meta: ModMeta) -> Result<ProjectInfo, String> {
    let dir = config::projects_root().join(sanitize_name(&meta.name));
    if dir.exists() {
        return Err(format!("项目已存在: {}", dir.display()));
    }
    for sub in ["characters", "reference", "artwork", "mod", "mod_compiled", "logs"] {
        std::fs::create_dir_all(dir.join(sub)).map_err(|e| format!("创建项目目录失败: {e}"))?;
    }
    let info = ProjectInfo {
        path: dir.to_string_lossy().into_owned(),
        name: meta.name.clone(),
        meta,
        created_at: now(),
        ..Default::default()
    };
    save(&info)?;
    Ok(info)
}

pub fn save(info: &ProjectInfo) -> Result<(), String> {
    std::fs::create_dir_all(Path::new(&info.path)).map_err(|e| format!("创建项目目录失败: {e}"))?;
    let s = serde_json::to_string_pretty(info).map_err(|e| format!("序列化项目失败: {e}"))?;
    std::fs::write(project_file(Path::new(&info.path)), s).map_err(|e| format!("写入项目失败: {e}"))
}

pub fn load(path: &str) -> Result<ProjectInfo, String> {
    let file = project_file(Path::new(path));
    let s = std::fs::read_to_string(&file).map_err(|e| format!("读取项目失败: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("项目文件解析失败: {e}"))
}

pub fn list() -> Result<Vec<ProjectInfo>, String> {
    let root = config::projects_root();
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && p.join("project.json").is_file() {
                if let Ok(info) = load(&p.to_string_lossy()) {
                    out.push(info);
                }
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

/// 取（或新建）某阶段状态
pub fn stage<'a>(info: &'a mut ProjectInfo, id: &str) -> &'a mut StageState {
    info.pipeline
        .entry(id.to_string())
        .or_insert_with(StageState::default)
}

/// 保存人设草稿（不冻结；Tab2 对话产出）
pub fn save_draft(info: &mut ProjectInfo, sheet: CharacterSheet) -> Result<(), String> {
    let errs = sheet.validate();
    if !errs.is_empty() {
        return Err(format!("草稿校验失败：{}", errs.join("；")));
    }
    let artifact = CharacterArtifact {
        version: "draft".into(),
        json: serde_json::to_value(&sheet).map_err(|e| e.to_string())?,
        markdown: sheet.to_markdown(),
        frozen: false,
        draft: true,
    };
    info.character = Some(artifact);
    save(info)
}

/// 定稿冻结：校验通过 → 写入 characters/ 文件 + 递增版本号
pub fn freeze_character(info: &mut ProjectInfo) -> Result<String, String> {
    let Some(ch) = &info.character else {
        return Err("尚未产出人设草稿，请先在 Tab2 与 Agent 对话".into());
    };
    let sheet: CharacterSheet =
        serde_json::from_value(ch.json.clone()).map_err(|e| format!("人设 JSON 解析失败: {e}"))?;
    let errs = sheet.validate();
    if !errs.is_empty() {
        return Err(errs.join("；"));
    }
    let next = next_version(ch.version.as_str());
    let dir = Path::new(&info.path);
    let char_dir = dir.join("characters");
    std::fs::create_dir_all(&char_dir).map_err(|e| format!("创建 characters 目录失败: {e}"))?;
    let json_path = char_dir.join(format!("character_{next}.json"));
    let md_path = char_dir.join(format!("character_{next}.md"));
    std::fs::write(
        &json_path,
        serde_json::to_string_pretty(&sheet).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("写入人设 JSON 失败: {e}"))?;
    std::fs::write(&md_path, sheet.to_markdown()).map_err(|e| format!("写入人设文档失败: {e}"))?;
    info.character = Some(CharacterArtifact {
        version: next.clone(),
        json: serde_json::to_value(&sheet).map_err(|e| e.to_string())?,
        markdown: sheet.to_markdown(),
        frozen: true,
        draft: false,
    });
    save(info)?;
    Ok(next)
}

/// 版本号递增：v1 → v2；draft → v1
fn next_version(cur: &str) -> String {
    let num: u32 = cur
        .rsplit(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    format!("v{}", num + 1)
}

/// 追加聊天记录（logs/chat.jsonl）
pub fn append_chat_log(path: &str, messages: &[crate::core::llm::ChatMsg], reply: &str) -> Result<(), String> {
    let logs_dir = Path::new(path).join("logs");
    std::fs::create_dir_all(&logs_dir).map_err(|e| format!("创建日志目录失败: {e}"))?;
    let file = logs_dir.join("chat.jsonl");
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .map_err(|e| format!("打开聊天记录失败: {e}"))?;
    for m in messages {
        let line = serde_json::json!({"role": m.role, "content": m.content, "ts": now()});
        writeln!(f, "{line}").map_err(|e| format!("写入聊天记录失败: {e}"))?;
    }
    writeln!(f, "{}", serde_json::json!({"role": "assistant", "content": reply, "ts": now()}))
        .map_err(|e| format!("写入聊天记录失败: {e}"))
}

/// 复制参考图到项目 reference/ 目录（文件名净化 + 冲突自动加序号），
/// 返回目录内全部参考图文件名。Tab2 上传用。
pub fn import_reference(project_path: &str, src: &str) -> Result<Vec<String>, String> {
    // 先做项目存在性校验，避免写入非法/越权路径
    let _info = load(project_path)?;
    let src_path = Path::new(src);
    if !src_path.is_file() {
        return Err(format!("参考图不存在：{src}"));
    }
    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| "png".to_string());
    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("reference");
    let base = sanitize_name(stem);
    let dir = Path::new(project_path).join("reference");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 reference 目录失败：{e}"))?;

    let mut target = dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while target.exists() {
        target = dir.join(format!("{base}_{n}.{ext}"));
        n += 1;
    }
    std::fs::copy(src_path, &target).map_err(|e| format!("复制参考图失败：{e}"))?;

    let mut names: Vec<String> = if let Ok(rd) = std::fs::read_dir(&dir) {
        rd.flatten()
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .collect()
    } else {
        vec![]
    };
    names.sort();
    Ok(names)
}
