//! 控制台调试命令生成（Tab4 / PROJECT_SPEC.md §4.4）
use crate::core::project::ProjectInfo;
use crate::core::schema::CharacterSheet;

pub fn gen_cmds(info: &ProjectInfo) -> Vec<String> {
    let mut cmds = vec![];
    if let Some(ch) = &info.character {
        if let Ok(sheet) = serde_json::from_value::<CharacterSheet>(ch.json.clone()) {
            cmds.push("-- 生成测试角色控制台命令（游戏内按 ~ 打开控制台粘贴执行）".to_string());
            for item in &sheet.starter_items {
                cmds.push(format!("c_give(\"{item}\", 1)"));
            }
            for it in &sheet.special_items {
                cmds.push(format!("c_give(\"{}\", 1)", it.id));
            }
            cmds.push("print(\"HP=\" .. ThePlayer.components.health.currenthealth)".to_string());
            cmds.push("print(\"HUNGER=\" .. ThePlayer.components.hunger.current)".to_string());
            cmds.push("print(\"SANITY=\" .. ThePlayer.components.sanity.current)".to_string());
        }
    }
    if cmds.is_empty() {
        cmds.push("-- 该项目尚未产出定稿人设，请先到 Tab2 完成人设对话。".to_string());
    }
    cmds
}