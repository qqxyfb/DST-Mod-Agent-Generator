//! Tab1 全局配置相关 command
//! 安全约束：API Key 仅本地保存；测试请求直连用户配置地址，不中转第三方
use crate::core::{config, dst, py};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvItem {
    pub name: String,
    pub ready: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvSetupResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub message: String,
}

#[tauri::command]
pub fn get_config() -> config::AppConfig {
    config::load()
}

#[tauri::command]
pub fn save_config(cfg: config::AppConfig) -> Result<config::AppConfig, String> {
    config::save(&cfg)?;
    Ok(cfg)
}

#[tauri::command]
pub fn validate_dst_dir(path: String) -> dst::DstInfo {
    dst::validate(&path)
}

/// LLM 连通性测试：发一个最小 chat 请求
#[tauri::command]
pub async fn llm_test(cfg: config::LlmConfig) -> TestResult {
    if cfg.base_url.trim().is_empty() || cfg.model.trim().is_empty() {
        return TestResult {
            ok: false,
            message: "请先填写 Base URL 与模型名称".into(),
        };
    }
    let client = crate::core::llm::LlmClient::new(cfg.clone());
    match client
        .chat(vec![crate::core::llm::ChatMsg {
            role: "user".into(),
            content: "请只回复 OK 两个字母。".into(),
        }])
        .await
    {
        Ok(_) => TestResult {
            ok: true,
            message: format!("连通正常（模型：{}）", cfg.model),
        },
        Err(e) => TestResult { ok: false, message: e },
    }
}

/// 图像接口测试：V1 预留（MVP 不接入）
#[tauri::command]
pub fn image_test(_cfg: config::ImageConfig) -> TestResult {
    TestResult {
        ok: false,
        message: "图像接口 V1 预留，尚未接入（OpenAI 兼容 images API 将在 V1 支持）".into(),
    }
}

/// 工具环境状态：按功能组探测就绪情况
#[tauri::command]
pub fn python_env_status() -> Vec<EnvItem> {
    let mut items = vec![];
    let bridge = py::PythonBridge::detect();

    match &bridge {
        Some(b) => items.push(EnvItem {
            name: "Python 环境".into(),
            ready: true,
            detail: format!("检测到 {}", b.python),
        }),
        None => items.push(EnvItem {
            name: "Python 环境".into(),
            ready: false,
            detail: "未检测到 python / py -3（安装 Python 3.10+ 后重试）".into(),
        }),
    }

    // 基础依赖（Pillow / numpy）：MVP 一键安装目标，刷新后可直观确认
    let pil = bridge.as_ref().and_then(|b| b.module_version("Pillow"));
    let np = bridge.as_ref().and_then(|b| b.module_version("numpy"));
    let base_ok = pil.is_some() && np.is_some();
    let base_detail = match (&pil, &np) {
        (Some(p), Some(n)) => format!("Pillow {p} / numpy {n}"),
        (Some(p), None) => format!("Pillow {p} 已装，numpy 缺失（点“一键安装基础依赖”）"),
        (None, Some(_)) => "Pillow 缺失，numpy 已装（点“一键安装基础依赖”）".into(),
        (None, None) => "未安装（点“一键安装基础依赖”安装 Pillow / numpy）".into(),
    };
    items.push(EnvItem {
        name: "基础依赖（Pillow / numpy）".into(),
        ready: base_ok,
        detail: base_detail,
    });
    let rembg = bridge.as_ref().map(|b| b.module_available("rembg")).unwrap_or(false);
    items.push(EnvItem {
        name: "rembg 抠图".into(),
        ready: rembg,
        detail: if rembg { "就绪".into() } else { "未安装（V1 一键安装自动安装）".into() },
    });

    let sam2 = bridge.as_ref().map(|b| b.module_available("sam2")).unwrap_or(false);
    items.push(EnvItem {
        name: "SAM2 分割".into(),
        ready: sam2,
        detail: if sam2 { "就绪".into() } else { "未安装 / 权重未下载（V1）".into() },
    });

    let cfg = config::load();
    let user_found = find_autocompiler(&cfg.modtools_path);
    let auto_found = user_found.clone().or_else(dst::find_steam_autocompiler);
    let (ac_ready, ac_detail) = match &auto_found {
        Some(p) if user_found.is_some() => (true, p.clone()),
        Some(p) => (true, format!("{p}（自动发现于 Steam 库，可点“自动检测”填入配置）")),
        None => (
            false,
            "未配置 Klei Mod Tools（已自动扫描 Steam 库未找到；请通过 Steam 安装，或手动填写路径）".into(),
        ),
    };
    items.push(EnvItem {
        name: "autocompiler.exe".into(),
        ready: ac_ready,
        detail: ac_detail,
    });

    let esc = config::repo_root().join("templates").join("esc");
    let esc_ok = esc.is_dir() && esc.join("README.md").exists();
    items.push(EnvItem {
        name: "ESC 模板".into(),
        ready: esc_ok,
        detail: if esc_ok {
            "已随包附带".into()
        } else {
            format!("未找到模板目录（{}），V2 随包附带", esc.display())
        },
    });

    items
}

/// 按功能组一键安装依赖（base / rembg / sam2）。
/// - base ：Pillow / numpy（MVP 必需）
/// - rembg：抠图（V1 图像管线，含 onnxruntime CPU 推理）
/// - sam2 ：分割（CPU 版 torch + sam2 包，体积大；分割权重由图像管线首次使用时下载）
/// mirror：可选 pip HTTPS 镜像。用户 pip.ini 若配置 http 源，新版 pip 会因
///         trusted-host 键迁移到 global 段而忽略该源，此处允许显式指定 https 镜像解决。
#[tauri::command]
pub async fn python_env_setup(
    group: Option<String>,
    mirror: Option<String>,
) -> Result<EnvSetupResult, String> {
    let group = group.unwrap_or_else(|| "base".into());
    if !matches!(group.as_str(), "base" | "rembg" | "sam2") {
        return Ok(EnvSetupResult {
            ok: false,
            message: format!("未知安装分组：{group}（支持 base / rembg / sam2）"),
        });
    }
    let Some(bridge) = py::PythonBridge::detect() else {
        return Ok(EnvSetupResult {
            ok: false,
            message: "未检测到 Python。请到 python.org 安装 Python 3.10+（安装时勾选 Add to PATH）后重试。".into(),
        });
    };
    let mut index_args: Vec<&str> = vec![];
    if let Some(m) = mirror.as_deref().map(str::trim) {
        if !m.is_empty() {
            index_args = vec!["--index-url", m];
        }
    }

    let msg = match group.as_str() {
        "base" => {
            let req = config::repo_root().join("python").join("requirements.txt");
            if !req.is_file() {
                return Ok(EnvSetupResult {
                    ok: false,
                    message: format!("未找到依赖清单：{}", req.display()),
                });
            }
            let out = tokio::process::Command::new(&bridge.python)
                .env("PYTHONIOENCODING", "utf-8")
                .arg("-m").arg("pip").arg("install").arg("--disable-pip-version-check")
                .args(index_args.clone())
                .arg("-r").arg(&req)
                .output().await.map_err(|e| format!("启动 pip 失败：{e}"))?;
            if !out.status.success() {
                return Ok(pip_failed(&out));
            }
            "基础依赖安装完成（Pillow / numpy），可点“刷新状态”确认。".to_string()
        }
        "rembg" => {
            let out = tokio::process::Command::new(&bridge.python)
                .env("PYTHONIOENCODING", "utf-8")
                .arg("-m").arg("pip").arg("install").arg("--disable-pip-version-check")
                .args(index_args.clone())
                .args(["rembg", "onnxruntime"])
                .output().await.map_err(|e| format!("启动 pip 失败：{e}"))?;
            if !out.status.success() {
                return Ok(pip_failed(&out));
            }
            "rembg 安装完成（含 onnxruntime CPU 推理）。抠图模型权重首次使用时自动下载（约 180MB）。".to_string()
        }
        "sam2" => {
            // 1) CPU 版 torch / torchvision（官方 CPU wheel 源，体积大）
            let out1 = tokio::process::Command::new(&bridge.python)
                .env("PYTHONIOENCODING", "utf-8")
                .arg("-m").arg("pip").arg("install").arg("--disable-pip-version-check")
                .args(["--index-url", "https://download.pytorch.org/whl/cpu"])
                .args(["torch", "torchvision"])
                .output().await.map_err(|e| format!("启动 pip 失败：{e}"))?;
            if !out1.status.success() {
                return Ok(pip_failed(&out1));
            }
            // 2) SAM2 包（PyPI：sam2>=1.1.0）
            let out2 = tokio::process::Command::new(&bridge.python)
                .env("PYTHONIOENCODING", "utf-8")
                .arg("-m").arg("pip").arg("install").arg("--disable-pip-version-check")
                .args(index_args)
                .args(["sam2"])
                .output().await.map_err(|e| format!("启动 pip 失败：{e}"))?;
            if !out2.status.success() {
                return Ok(pip_failed(&out2));
            }
            "SAM2（CPU 版）安装完成。分割权重（sam2.1_hiera，约 75MB~900MB）由图像流水线首次使用时下载。".to_string()
        }
        _ => unreachable!(),
    };
    Ok(EnvSetupResult { ok: true, message: msg })
}

/// pip 失败信息（截断 stderr，编码统一 UTF-8）
fn pip_failed(out: &std::process::Output) -> EnvSetupResult {
    EnvSetupResult {
        ok: false,
        message: format!(
            "pip 安装失败：{}",
            config::truncate(&String::from_utf8_lossy(&out.stderr), 400)
        ),
    }
}

/// 自动检测 autocompiler.exe（Steam 注册表安装路径 + libraryfolders.vdf 全部库）
#[tauri::command]
pub fn detect_modtools() -> Option<String> {
    crate::core::dst::find_steam_autocompiler()
}

/// 在配置路径下查找 autocompiler.exe（文件本身或最多 3 层递归目录）
fn find_autocompiler(root: &str) -> Option<String> {
    let p = Path::new(root);
    if p.is_file() {
        if p.file_name().and_then(|n| n.to_str()) == Some("autocompiler.exe") {
            return Some(p.to_string_lossy().into_owned());
        }
        return None;
    }
    if !p.is_dir() {
        return None;
    }
    fn walk(d: &Path, depth: usize) -> Option<String> {
        if depth > 3 {
            return None;
        }
        let rd = std::fs::read_dir(d).ok()?;
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Some(f) = walk(&p, depth + 1) {
                    return Some(f);
                }
            } else if p.file_name().and_then(|n| n.to_str()) == Some("autocompiler.exe") {
                return Some(p.to_string_lossy().into_owned());
            }
        }
        None
    }
    walk(p, 0)
}

/// GitHub Release 版本检查（配置了仓库才请求；MVP 仅提示，不自动替换）
#[tauri::command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let cfg = config::load();
    let cur = config::CURRENT_VERSION.to_string();
    let repo = cfg.update.repo.trim().to_string();
    if repo.is_empty() {
        return Ok(UpdateInfo {
            current_version: cur,
            latest_version: None,
            update_available: false,
            message: "未配置 GitHub 仓库地址（Tab1 版本更新设置）".into(),
        });
    }
    // 清洗仓库输入：支持 owner/repo 或完整 URL
    let cleaned = repo
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_matches('/')
        .to_string();
    let url = format!("https://api.github.com/repos/{cleaned}/releases/latest");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;
    let resp = client
        .get(&url)
        .header("User-Agent", "DST-Mod-Agent-Generator")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await;
    match resp {
        Ok(r) if r.status().is_success() => {
            let v: Value = r
                .json()
                .await
                .map_err(|e| format!("解析 Release 响应失败: {e}"))?;
            let tag = v["tag_name"]
                .as_str()
                .unwrap_or_default()
                .trim_start_matches('v')
                .to_string();
            if tag.is_empty() {
                return Ok(UpdateInfo {
                    current_version: cur,
                    latest_version: None,
                    update_available: false,
                    message: "仓库未发布任何 Release".into(),
                });
            }
            let available = tag != cur;
            let message = if available {
                format!("发现新版本 v{tag}（当前 v{cur}）。自动替换流程（V1）：下载 → 退出后替换 → 重启。")
            } else {
                format!("当前已是最新版本 v{cur}")
            };
            Ok(UpdateInfo {
                current_version: cur,
                latest_version: Some(tag),
                update_available: available,
                message,
            })
        }
        Ok(r) => {
            let code = r.status();
            let body = r.text().await.unwrap_or_default();
            Ok(UpdateInfo {
                current_version: cur,
                latest_version: None,
                update_available: false,
                message: format!("查询失败 HTTP {code}: {}", config::truncate(&body, 200)),
            })
        }
        Err(e) => Ok(UpdateInfo {
            current_version: cur,
            latest_version: None,
            update_available: false,
            message: format!("网络错误: {e}"),
        }),
    }
}
