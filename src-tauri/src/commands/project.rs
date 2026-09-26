//! Tab2 项目创建与人设对话相关 command
use crate::core::{config, llm, project, schema};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AgentReply {
    pub reply: String,
    pub draft: Option<schema::CharacterSheet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub ok: bool,
    pub errors: Vec<String>,
}

#[tauri::command]
pub fn create_project(meta: project::ModMeta, notes: String) -> Result<project::ProjectInfo, String> {
    project::create(meta, notes)
}

#[tauri::command]
pub fn list_projects() -> Result<Vec<project::ProjectInfo>, String> {
    project::list()
}

#[tauri::command]
pub fn open_project(path: String) -> Result<project::ProjectInfo, String> {
    project::load(&path)
}

/// 保存 Mod 基础信息 + Tab2 角色描述文本（Tab2「保存修改」）
#[tauri::command]
pub fn update_project(
    path: String,
    meta: project::ModMeta,
    notes: String,
) -> Result<project::ProjectInfo, String> {
    project::update(&path, meta, notes)
}

/// 列出项目 reference/ 目录内已导入的参考图文件名（Tab2 切换项目时回填）
#[tauri::command]
pub fn list_references(project_path: String) -> Result<Vec<String>, String> {
    project::list_references(&project_path)
}

/// 导入参考图到项目 reference/ 目录（Tab2 上传）
#[tauri::command]
pub fn import_reference(project_path: String, file_path: String) -> Result<Vec<String>, String> {
    project::import_reference(&project_path, &file_path)
}

/// 人设对话：调用 LLM 迭代角色设定；回复中带完整 JSON 时产出草稿并保存
#[tauri::command]
pub async fn agent_chat(
    project_path: String,
    messages: Vec<llm::ChatMsg>,
) -> Result<AgentReply, String> {
    let _info = project::load(&project_path)?;
    let cfg = config::load();
    if cfg.llm.base_url.trim().is_empty() || cfg.llm.model.trim().is_empty() {
        return Ok(AgentReply {
            reply: "请先在 Tab1 配置 LLM API（Base URL / 模型名称），再开始人设对话。".into(),
            draft: None,
        });
    }

    let client = llm::LlmClient::new(cfg.llm.clone());
    let system = format!(
        "你是《饥荒：联机版》(Don't Starve Together) 的人物 Mod 人设顾问，根据用户的中文描述逐步提炼完整角色设定。\n\
         {}\n\
         对话规则：\n\
         1. 信息不足时用中文简短提问（每次最多 3 个问题），不要替用户编造关键设定。\n\
         2. 当信息足以定稿时，回复 = 一段中文总结 + 完整 JSON 草稿（严格符合上述结构，不要多余字段）。\n\
         3. 三围填 50~300 的整数，未知可默认 150；starter_items 使用 DST 原生物品名，最多 6 个；专属道具放入 special_items。",
        schema::json_schema_hint()
    );
    let reply = match client
        .chat({
            let mut msgs = vec![llm::ChatMsg {
                role: "system".into(),
                content: system,
            }];
            msgs.extend(messages.clone());
            msgs
        })
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Ok(AgentReply {
                reply: format!("⚠ LLM 调用失败：{e}"),
                draft: None,
            })
        }
    };
    let _ = project::append_chat_log(&project_path, &messages, &reply);

    // 尝试从回复提取人设 JSON：有效则保存草稿
    let draft = llm::extract_json(&reply)
        .and_then(|v| serde_json::from_value::<schema::CharacterSheet>(v).ok());
    if let Some(sheet) = draft {
        let errs = sheet.validate();
        if errs.is_empty() {
            let mut info = project::load(&project_path)?;
            project::save_draft(&mut info, sheet.clone())?;
            return Ok(AgentReply {
                reply,
                draft: Some(sheet),
            });
        }
        return Ok(AgentReply {
            reply: format!("{reply}\n\n⚠ 草稿校验未通过：{}", errs.join("；")),
            draft: None,
        });
    }
    Ok(AgentReply { reply, draft: None })
}

/// 确认人设定稿：校验通过后冻结为 vN 版本快照
#[tauri::command]
pub fn confirm_character(project_path: String) -> Result<ValidationResult, String> {
    let mut info = project::load(&project_path)?;
    match project::freeze_character(&mut info) {
        Ok(ver) => Ok(ValidationResult {
            ok: true,
            errors: vec![format!("已冻结人设版本 {ver}")],
        }),
        Err(e) => Ok(ValidationResult {
            ok: false,
            errors: vec![e],
        }),
    }
}
