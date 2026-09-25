//! Tab3 流水线（阶段状态机）相关 command
use crate::core::pipeline;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModReport {
    #[serde(default)]
    pub mod_dir: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

/// 阶段状态总览（pipeline map）
#[tauri::command]
pub fn pipeline_state(project_path: String) -> Result<Value, String> {
    pipeline::state(&project_path)
}

/// 开始执行阶段（视觉线/代码线可并行；共享 Stage1 产物）
#[tauri::command]
pub async fn start_stage(
    project_path: String,
    stage_id: String,
    app: AppHandle,
) -> Result<Value, String> {
    pipeline::run_stage(app, &project_path, &stage_id).await
}

/// 重跑阶段 = 基于当前输入快照重新执行（用户补充需求：单阶段可重新生成）
#[tauri::command]
pub async fn rerun_stage(
    project_path: String,
    stage_id: String,
    app: AppHandle,
) -> Result<Value, String> {
    pipeline::run_stage(app, &project_path, &stage_id).await
}

/// 跳过阶段（标记风险，可恢复执行）
#[tauri::command]
pub fn skip_stage(project_path: String, stage_id: String) -> Result<Value, String> {
    pipeline::skip(&project_path, &stage_id)?;
    pipeline::state(&project_path)
}

/// 某阶段日志文本
#[tauri::command]
pub fn get_stage_log(project_path: String, stage_id: String) -> Result<String, String> {
    pipeline::stage_log(&project_path, &stage_id)
}

/// 生成/读取 Mod 报告（Stage7）
#[tauri::command]
pub fn get_report(project_path: String) -> Result<ModReport, String> {
    let v = pipeline::report(&project_path)?;
    serde_json::from_value(v).map_err(|e| format!("报告解析失败: {e}"))
}
