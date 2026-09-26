//! 运行时日志模块：追加写入项目根 log\runtime.log
//! 目录不可写时（如安装目录受保护）fallback 到 %LOCALAPPDATA%\com.dstmod.agent\log。
//! 设计约束：不引入第三方日志库，保持轻量，每次调用单行追加。

use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// 日志目录：优先 `<cwd>\log`，失败则 fallback 到 %LOCALAPPDATA%
pub fn log_dir() -> PathBuf {
    if let Ok(d) = std::env::current_dir() {
        let p = d.join("log");
        if fs::create_dir_all(&p).is_ok() {
            return p;
        }
    }
    let fallback = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.dstmod.agent")
        .join("log");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

/// 追加一行运行日志：`[yyyy-MM-dd HH:mm:ss] [模块] 消息`
pub fn log(module: &str, msg: &str) {
    let line = format!(
        "[{}] [{}] {}\r\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        module,
        msg
    );
    let path = log_dir().join("runtime.log");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// 应用启动时调用：写启动日志并清理 30 天前的 runtime*.log
pub fn init() {
    log("app", "DST-Mod-Agent-Generator started");
    if let Ok(rd) = fs::read_dir(log_dir()) {
        let cutoff = chrono::Duration::days(30);
        let now = Local::now();
        for e in rd.flatten() {
            let is_log = e.path().extension().map(|x| x == "log").unwrap_or(false);
            if !is_log {
                continue;
            }
            if let Ok(meta) = e.metadata() {
                if let Ok(modified) = meta.modified() {
                    let t = chrono::DateTime::<chrono::Local>::from(modified);
                    if (now - t) > cutoff {
                        let _ = fs::remove_file(e.path());
                    }
                }
            }
        }
    }
}