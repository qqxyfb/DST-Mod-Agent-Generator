//! DST-Mod-Agent-Generator 主程序入口
//! 模块划分（见 PROJECT_SPEC.md §7.2）：
//! - commands: Tauri command 层（GUI 调用的后端接口，薄封装）
//! - core:     业务逻辑（配置/项目/流水线/LLM/日志等）

pub mod commands;
pub mod core;

/// Tauri 应用启动：注册全部 command。
pub fn run() {
    // 初始化运行时日志（log\runtime.log）
    crate::core::runtime_log::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Tab1 配置
            commands::config::get_config,
            commands::config::save_config,
            commands::config::validate_dst_dir,
            commands::config::llm_test,
            commands::config::image_test,
            commands::config::python_env_status,
            commands::config::python_env_setup,
            commands::config::detect_modtools,
            commands::config::check_update,
            // Tab2 项目与人设
            commands::project::create_project,
            commands::project::list_projects,
            commands::project::open_project,
            commands::project::update_project,
            commands::project::list_references,
            commands::project::import_reference,
            commands::project::agent_chat,
            commands::project::confirm_character,
            // Tab3 流水线
            commands::pipeline::pipeline_state,
            commands::pipeline::start_stage,
            commands::pipeline::rerun_stage,
            commands::pipeline::skip_stage,
            commands::pipeline::get_stage_log,
            commands::pipeline::get_report,
            // Tab4 调试
            commands::debug::locate_logs,
            commands::debug::read_log,
            commands::debug::fix_from_log,
            commands::debug::gen_console_cmds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}