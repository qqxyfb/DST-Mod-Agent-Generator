//! Rust → Python 子进程桥（预留接口，V1 图像管线接入）
//! 协议：stdio JSONL（request/progress/result/cancel），服务端实现见 python/bridge.py
//! 限制固化（PROJECT_SPEC.md §12）：本桥只承载图像处理（rembg/SAM2/后处理），
//! 不实现骨骼动画生成与 ComfyUI 工作流编排。
use serde_json::Value;
use std::process::Command;

pub struct PythonBridge {
    pub python: String,
}

impl PythonBridge {
    /// 探测可用的 Python（python 或 py -3）
    pub fn detect() -> Option<PythonBridge> {
        for (cand, args) in [("python", vec!["--version"]), ("py", vec!["-3", "--version"])] {
            if let Ok(out) = Command::new(cand).args(&args).output() {
                if out.status.success() {
                    return Some(PythonBridge { python: cand.to_string() });
                }
            }
        }
        None
    }

    pub fn module_available(&self, module: &str) -> bool {
        let script = format!(
            "import importlib.util; print(importlib.util.find_spec('{module}') is not None)"
        );
        match Command::new(&self.python).arg("-c").arg(&script).output() {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim() == "True"
            }
            _ => false,
        }
    }

    /// 查询已安装发行版版本（importlib.metadata），未安装返回 None
    pub fn module_version(&self, dist: &str) -> Option<String> {
        let script = format!("import importlib.metadata as m; print(m.version(\"{dist}\"))");
        match Command::new(&self.python).arg("-c").arg(&script).output() {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            }
            _ => None,
        }
    }
    /// V1：调用图像管线命令
    /// fn run(&self, cmd: &str, args: Value) -> Result<Value, String> { todo!() }
    #[allow(dead_code)]
    pub fn _run_placeholder(&self, _cmd: &str, _args: Value) -> Result<Value, String> {
        Err("Python 图像管线 V1 未接入（预留接口）".to_string())
    }
}