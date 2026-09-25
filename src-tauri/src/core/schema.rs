//! 人设 JSON Schema 校验（PROJECT_SPEC.md §7.5）
//! 白名单 + 数值范围强校验，防止 LLM 输出非法 prefab 参数
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub health: i64,
    pub hunger: i64,
    pub sanity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    pub id: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub item: Option<String>,
    #[serde(default)]
    pub cooldown: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemDef {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub stats: Option<Stats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProsCons {
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterSheet {
    pub char_name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub stats: Stats,
    #[serde(default)]
    pub passive_skills: Vec<Skill>,
    #[serde(default)]
    pub active_skills: Vec<Skill>,
    #[serde(default)]
    pub starter_items: Vec<String>,
    #[serde(default)]
    pub special_items: Vec<ItemDef>,
    #[serde(default)]
    pub speech: HashMap<String, String>,
    #[serde(default)]
    pub pros_cons: ProsCons,
}

pub const STAT_MIN: i64 = 50;
pub const STAT_MAX: i64 = 300;

impl CharacterSheet {
    /// 返回全部校验错误（空数组 = 通过）
    pub fn validate(&self) -> Vec<String> {
        let mut errs = vec![];
        if self.char_name.is_empty() {
            errs.push("char_name 不能为空".into());
        }
        let valid_name = !self.char_name.is_empty()
            && self
                .char_name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_name {
            errs.push("char_name 必须为小写字母/数字/下划线（如 wanda_cn）".into());
        }
        if self.display_name.is_empty() {
            errs.push("display_name 不能为空".into());
        }
        for (k, v) in [
            ("health", self.stats.health),
            ("hunger", self.stats.hunger),
            ("sanity", self.stats.sanity),
        ] {
            if !(STAT_MIN..=STAT_MAX).contains(&v) {
                errs.push(format!("{k} 超出范围 {STAT_MIN}~{STAT_MAX}: {v}"));
            }
        }
        if self.starter_items.len() > 6 {
            errs.push(format!("starter_items 最多 6 项（当前 {}）", self.starter_items.len()));
        }
        if self.starter_items.iter().any(|s| s.trim().is_empty()) {
            errs.push("starter_items 存在空项".into());
        }
        errs
    }

    /// 生成人设 Markdown 文档
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}（{}）\n\n", self.display_name, self.char_name));
        if !self.description.is_empty() {
            md.push_str(&format!("> {}\n\n", self.description));
        }
        let s = &self.stats;
        md.push_str(&format!(
            "## 三围\n- 生命（health）：{}\n- 饥饿（hunger）：{}\n- 精神（sanity）：{}\n\n",
            s.health, s.hunger, s.sanity
        ));
        md.push_str("## 被动技能\n");
        for sk in &self.passive_skills {
            md.push_str(&format!("- {}（params: {}）\n", sk.id, sk.params));
        }
        md.push_str("\n## 主动技能\n");
        for sk in &self.active_skills {
            let item = sk.item.as_deref().unwrap_or("-");
            let cd = sk.cooldown.map(|c| c.to_string()).unwrap_or_else(|| "-".into());
            md.push_str(&format!("- {} 载体={} 冷却={}\n", sk.id, item, cd));
        }
        md.push_str("\n## 开局物品\n");
        for it in &self.starter_items {
            md.push_str(&format!("- {it}\n"));
        }
        if !self.special_items.is_empty() {
            md.push_str("\n## 专属道具\n");
            for it in &self.special_items {
                md.push_str(&format!("- {}（{}）\n", it.id, it.kind));
            }
        }
        md.push_str("\n## 台词（节选）\n");
        for (k, v) in self.speech.iter().take(12) {
            md.push_str(&format!("- {k}: {v}\n"));
        }
        md.push_str("\n## 优缺点\n");
        for p in &self.pros_cons.pros {
            md.push_str(&format!("- 优点：{p}\n"));
        }
        for c in &self.pros_cons.cons {
            md.push_str(&format!("- 缺点：{c}\n"));
        }
        md
    }
}

/// 给 LLM 的 JSON 结构提示（与 CharacterSheet 对应）
pub fn json_schema_hint() -> String {
    r#"
JSON 结构（严格遵循，不要输出多余字段）：
{
  "char_name": "小写snake_case英文名",
  "display_name": "中文显示名",
  "description": "一句话简介（可选）",
  "stats": { "health": 150, "hunger": 150, "sanity": 150 },
  "passive_skills": [{ "id": "技能ID", "params": {} }],
  "active_skills": [{ "id": "主动技能ID", "item": "专属道具prefab名", "cooldown": 120 }],
  "starter_items": ["axe", "torch"],
  "special_items": [{ "id": "item_id", "kind": "wearable|consumable|weapon", "stats": {} }],
  "speech": { "ANNOUNCE_ACCOMPLISHMENT": "中文台词" },
  "pros_cons": { "pros": ["优点"], "cons": ["缺点"] }
}
约束：三围整数 50~300；char_name 只能是小写字母/数字/下划线；starter_items 最多 6 个。
"#
    .to_string()
}

/// JSON → Lua 字面量（生成 modinfo/modmain 埋点用；对象键按字符串处理）
pub fn json_to_lua(v: &Value) -> String {
    match v {
        Value::Null => "nil".into(),
        Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        Value::Number(n) => n.to_string(),
        // Rust Debug 转义与 Lua 字符串字面量基本兼容（\n \\ \" 等）
        Value::String(s) => format!("{:?}", s),
        Value::Array(a) => {
            let inner: Vec<String> = a.iter().map(json_to_lua).collect();
            format!("{{ {} }}", inner.join(", "))
        }
        Value::Object(o) => {
            let inner: Vec<String> = o
                .iter()
                .map(|(k, vv)| format!("[\"{}\"] = {}", k, json_to_lua(vv)))
                .collect();
            format!("{{ {} }}", inner.join(", "))
        }
    }
}