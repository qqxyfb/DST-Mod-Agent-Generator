//! DST 游戏目录校验与原版脚本白名单读取（PROJECT_SPEC.md §4.1.1 / §7.8）
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct DstInfo {
    pub valid: bool,
    pub errors: Vec<String>,
    pub scripts_count: usize,
    pub sample_scripts: Vec<String>,
}

/// 校验 DST 根目录：必须存在 mods/ 与 scripts/
pub fn validate(root: &str) -> DstInfo {
    let path = Path::new(root);
    if !path.is_dir() {
        return DstInfo {
            valid: false,
            errors: vec![format!("目录不存在: {root}")],
            scripts_count: 0,
            sample_scripts: vec![],
        };
    }
    let mut errors = vec![];
    for sub in ["mods", "scripts"] {
        if !path.join(sub).is_dir() {
            errors.push(format!("缺少 {sub}/ 文件夹"));
        }
    }
    let scripts = match read_whitelist(root) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e);
            vec![]
        }
    };
    let scripts_count = scripts.len();
    let sample_scripts = scripts
        .iter()
        .take(5)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    DstInfo {
        valid: errors.is_empty(),
        errors,
        scripts_count,
        sample_scripts,
    }
}

/// 白名单：关键人物 prefab + 常用组件（控制参考上下文 token 预算）
const WHITELIST: [&str; 9] = [
    "prefabs/wilson.lua",
    "prefabs/wolfgang.lua",
    "prefabs/wickerbottom.lua",
    "prefabs/wes.lua",
    "components/health.lua",
    "components/hunger.lua",
    "components/sanity.lua",
    "components/inventory.lua",
    "components/combat.lua",
];

pub fn read_whitelist(root: &str) -> Result<Vec<PathBuf>, String> {
    let base = Path::new(root).join("scripts");
    if !base.is_dir() {
        return Err(format!("scripts/ 目录不存在: {}", base.display()));
    }
    let mut out = vec![];
    for rel in WHITELIST {
        let full = base.join(rel);
        if full.is_file() {
            out.push(full);
        }
    }
    Ok(out)
}

/// ---------- Steam 工具自动发现（autocompiler.exe） ----------
/// 定位逻辑：1) 用户配置路径；2) 扫描 Steam 注册表安装路径与 libraryfolders.vdf 的全部库。
/// 限制固化：Klei Mod Tools 仅由 Steam 免费分发（无官方独立直链），此处只做本地检测，不实现下载安装。

/// 在指定目录下递归查找目标文件（depth 层内），返回完整路径
fn find_file(dir: &Path, name: &str, depth: usize) -> Option<String> {
    if depth == 0 || !dir.is_dir() {
        return None;
    }
    let rd = std::fs::read_dir(dir).ok()?;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(f) = find_file(&p, name, depth - 1) {
                return Some(f);
            }
        } else if p.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    None
}

/// 解析 Steam libraryfolders.vdf 中的库根路径（"path" 字段，含 \\ 转义）
fn parse_vdf_paths(text: &str) -> Vec<String> {
    let mut out = vec![];
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("\"path\"") {
            let rest = rest.trim();
            if let Some(inner) = rest.strip_prefix('"') {
                if let Some(end) = inner.find('"') {
                    let raw = &inner[..end];
                    out.push(raw.replace("\\\\", "\\"));
                }
            }
        }
    }
    out
}

/// 自动定位 autocompiler.exe：注册表 Steam 安装路径 + 默认路径 + libraryfolders.vdf 全部库
pub fn find_steam_autocompiler() -> Option<String> {
    let mut steam_dirs: Vec<PathBuf> = vec![];
    // 1) 注册表（读取为 UTF-8，避免 reg query 输出 GBK 乱码）
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        for sub in [r"SOFTWARE\WOW6432Node\Valve\Steam", r"SOFTWARE\Valve\Steam"] {
            if let Ok(k) = hklm.open_subkey(sub) {
                if let Ok(v) = k.get_value::<String, _>("InstallPath") {
                    steam_dirs.push(PathBuf::from(v));
                }
            }
        }
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(k) = hkcu.open_subkey(r"SOFTWARE\Valve\Steam") {
            if let Ok(v) = k.get_value::<String, _>("SteamPath") {
                steam_dirs.push(PathBuf::from(v));
            }
        }
    }
    // 2) 常见默认安装路径
    if let Ok(pf) = std::env::var("ProgramFiles(x86)") {
        steam_dirs.push(PathBuf::from(format!("{pf}\\Steam")));
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        steam_dirs.push(PathBuf::from(format!("{pf}\\Steam")));
    }

    // 3) 汇总候选库根（Steam 根 + 各 libraryfolders.vdf 声明的库）
    let mut roots: Vec<PathBuf> = vec![];
    for steam in steam_dirs {
        roots.push(steam.clone());
        let vdf = steam.join("steamapps").join("libraryfolders.vdf");
        if let Ok(text) = std::fs::read_to_string(&vdf) {
            for lib in parse_vdf_paths(&text) {
                roots.push(PathBuf::from(&lib));
            }
        }
    }

    // 4) 每个库的 steamapps\common\Don't Starve Mod Tools 下递归查找
    for root in roots {
        let modtools = root.join("steamapps").join("common").join("Don't Starve Mod Tools");
        if let Some(exe) = find_file(&modtools, "autocompiler.exe", 4) {
            return Some(exe);
        }
    }
    None
}

/// 拼接白名单脚本内容（受限长度），供 LLM 参考上下文注入
pub fn whitelist_snippets(root: &str, max_chars: usize, per_file: usize) -> Result<String, String> {
    let files = read_whitelist(root)?;
    let mut buf = String::new();
    for f in files {
        if buf.len() >= max_chars {
            break;
        }
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.lua")
            .to_string();
        if let Ok(content) = std::fs::read_to_string(&f) {
            let cut = content.chars().take(per_file).collect::<String>();
            buf.push_str(&format!("\n--- {name} ---\n{cut}\n"));
        }
    }
    Ok(buf)
}

pub fn scripts_index_json(root: &str) -> Result<Vec<String>, String> {
    let files = read_whitelist(root)?;
    Ok(files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}
