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

/// 读取项目对话历史（logs/chat.jsonl）：程序重启 / 切换项目后回填 Tab2 对话区；
/// 旧项目没有该文件时返回空数组，前端退回占位提示。
#[tauri::command]
pub fn load_chat_history(project_path: String) -> Result<Vec<llm::ChatMsg>, String> {
    project::load_chat(&project_path)
}

/// 人设顾问系统提示（agent_chat / agent_init 共用）
fn system_prompt() -> String {
    format!(
        "你是《饥荒：联机版》(Don't Starve Together) 的人物 Mod 人设顾问，根据用户的中文描述逐步提炼完整角色设定。\n\
         {}\n\
         对话规则：\n\
         1. 信息不足时用中文简短提问（每次最多 3 个问题），不要替用户编造关键设定。\n\
         2. 当信息足以定稿时，回复 = 一段中文总结 + 完整 JSON 草稿（严格符合上述结构，不要多余字段）。\n\
         3. 三围填 50~300 的整数，未知可默认 150；starter_items 使用 DST 原生物品名，最多 6 个；专属道具放入 special_items。\n\
         4. 本工具只复用 ESC 模板骨骼动作，不生成 Spriter 关键帧动画；不要输出动画/骨骼相关内容。",
        schema::json_schema_hint()
    )
}

/// 统一的 LLM 调用 + 草稿落盘流程（agent_chat / agent_init 共用）
async fn run_agent(project_path: &str, messages: Vec<llm::ChatMsg>) -> Result<AgentReply, String> {
    let _info = project::load(project_path)?;
    let cfg = config::load();
    if cfg.llm.base_url.trim().is_empty() || cfg.llm.model.trim().is_empty() {
        return Ok(AgentReply {
            reply: "请先点右上角「设置」配置 LLM API（Base URL / 模型名称），再开始人设对话。".into(),
            draft: None,
        });
    }

    let client = llm::LlmClient::new(cfg.llm.clone());
    let reply = match client
        .chat({
            let mut msgs = vec![llm::ChatMsg {
                role: "system".into(),
                content: system_prompt(),
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
    let _ = project::append_chat_log(project_path, &messages, &reply);

    // 尝试从回复提取人设 JSON：有效则保存草稿
    let draft = llm::extract_json(&reply)
        .and_then(|v| serde_json::from_value::<schema::CharacterSheet>(v).ok());
    if let Some(sheet) = draft {
        let errs = sheet.validate();
        if errs.is_empty() {
            let mut info = project::load(project_path)?;
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

/// 人设对话：调用 LLM 迭代角色设定；回复中带完整 JSON 时产出草稿并保存
#[tauri::command]
pub async fn agent_chat(
    project_path: String,
    messages: Vec<llm::ChatMsg>,
) -> Result<AgentReply, String> {
    run_agent(&project_path, messages).await
}

/// 一键初始化：不需要用户手写第一句，按程序内置的固定模板自动发起第一轮 Agent 请求，
/// 输入 = Mod 基础信息 + 用户描述（project.json 的 notes）+ 参考图文件名，
/// 输出 = 游戏内人物概设 + 人设 JSON 草稿（作为对话区的第一条消息）。
#[tauri::command]
pub async fn agent_init(project_path: String) -> Result<AgentReply, String> {
    let info = project::load(&project_path)?;
    let refs = project::list_references(&project_path)?;
    let notes = info.notes.trim().to_string();
    if notes.is_empty() && refs.is_empty() {
        return Ok(AgentReply {
            reply: "请先填写「角色人设 / 性格 / 技能想法」（可选：导入参考图），再点「一键初始化」。".into(),
            draft: None,
        });
    }
    let prompt = build_init_prompt(&info.meta, &notes, &refs);
    // 一键初始化 = 重开一轮人设推演：清空旧日志，保证「重新打开程序回填的对话」与本次结果一致
    project::reset_chat_log(&project_path)?;
    run_agent(
        &project_path,
        vec![llm::ChatMsg {
            role: "user".into(),
            content: prompt,
        }],
    )
    .await
}

/// 一键初始化的固定首轮提示词（模板固定在程序内，用户无需自己写第一句话）
fn build_init_prompt(meta: &project::ModMeta, notes: &str, refs: &[String]) -> String {
    let tags = if meta.tags.is_empty() {
        "（未选择）".to_string()
    } else {
        meta.tags.join("、")
    };
    let notes = if notes.is_empty() { "（未填写）" } else { notes };
    let refs = if refs.is_empty() {
        "（未导入参考图）".to_string()
    } else {
        format!("{}（MVP 阶段图像内容不解析，仅作命名与后续 V1 视觉模型的输入）", refs.join("、"))
    };
    format!(
        "【一键初始化 · 第 1 轮】请基于以下资料，一次性产出该角色在《饥荒：联机版》中的完整游戏内概设，并按系统提示的 JSON 结构给出人设草稿。\n\n\
         一、Mod 基础信息\n\
         - Mod 名称：{name}\n\
         - 作者：{author}\n\
         - 版本：{version}\n\
         - 简介：{desc}\n\
         - 创意工坊标签：{tags}\n\n\
         二、用户描述（角色人设 / 性格 / 技能想法）\n{notes}\n\n\
         三、参考图清单\n{refs}\n\n\
         四、输出要求（严格按此顺序）\n\
         1. 先用中文输出「游戏内人物概设」，依次包含：一句话定位、三围（生命/饥饿/精神）、被动技能、主动技能、开局物品、专属道具、性格与台词风格、优缺点。\n\
         2. 概设之后附上完整 JSON 人设草稿（严格符合系统提示中的 JSON 结构，不要多余字段）。\n\
         3. char_name 依据 Mod 名称推导（小写字母/数字/下划线），display_name 使用中文名。\n\
         4. 三围取 50~300 的整数，用户描述未提及时默认 150；starter_items 最多 6 个且使用 DST 原生物品名。\n\
         5. 用户描述含糊之处按合理推断给出，并在概设中标注「待确认」，不要臆造像素级美术细节。\n\
         6. 不要输出任何 Spriter / 骨骼动画相关内容（本工具只复用 ESC 模板骨骼动作）。",
        name = &meta.name,
        author = &meta.author,
        version = &meta.version,
        desc = &meta.description,
        tags = &tags,
        notes = notes,
        refs = &refs,
    )
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
