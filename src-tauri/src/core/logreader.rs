//! DST 游戏日志定位与读取（PROJECT_SPEC.md §8.4）
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize)]
pub struct LogFile {
    pub path: String,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogContent {
    pub path: String,
    pub lines: Vec<String>,
    pub error_indices: Vec<usize>,
}

/// 自动定位日志：Documents/Klei/DoNotStarveTogether 递归扫描（含分支子目录）
pub fn locate() -> Vec<LogFile> {
    let mut out = vec![];
    if let Some(docs) = dirs::document_dir() {
        let klei = docs.join("Klei");
        collect_logs(&klei, &mut out);
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out.truncate(10);
    out
}

fn collect_logs(dir: &Path, out: &mut Vec<LogFile>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_logs(&p, out);
        } else if p
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_lowercase().contains("log") && p.extension().and_then(|x| x.to_str()) == Some("txt"))
            .unwrap_or(false)
        {
            let modified = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t: SystemTime| {
                    let dt: DateTime<Utc> = t.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_default();
            out.push(LogFile {
                path: p.to_string_lossy().into_owned(),
                modified,
            });
        }
    }
}

/// 读取日志：UTF-8/GBK 宽松解码 + 标记错误行
pub fn read(path: &str) -> Result<LogContent, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("读取日志失败: {e}"))?;
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let error_indices = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            let low = l.to_lowercase();
            low.contains("error") || low.contains("stack traceback") || low.contains("attempt to")
        })
        .map(|(i, _)| i)
        .collect();
    Ok(LogContent {
        path: path.to_string(),
        lines,
        error_indices,
    })
}

/// 取最近 N 行（用于一键修复上下文）
pub fn tail(path: &str, n: usize) -> Result<String, String> {
    let c = read(path)?;
    let start = c.lines.len().saturating_sub(n);
    Ok(c.lines[start..].join("\n"))
}