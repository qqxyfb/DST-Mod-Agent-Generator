//! Tauri command 层：GUI 调用的后端接口（薄封装，只做参数校验与调用 core）
//! 与 src/api/tauri.ts 的 invoke 名称一一对应

pub mod config;
pub mod debug;
pub mod pipeline;
pub mod project;