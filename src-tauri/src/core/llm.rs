//! OpenAI 兼容 LLM 客户端（ollama / deepseek / qwen 等）
//! 安全约束：只向用户配置的 base_url 发请求，不中转第三方服务器
use crate::core::config::LlmConfig;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub role: String,
    pub content: String,
}

pub struct LlmClient {
    cfg: LlmConfig,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(cfg: LlmConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .unwrap_or_default();
        Self { cfg, http }
    }

    fn endpoint(&self) -> String {
        let base = self.cfg.base_url.trim_end_matches('/').to_string();
        if base.ends_with("/chat/completions") {
            base
        } else {
            format!("{base}/chat/completions")
        }
    }

    /// 普通对话
    pub async fn chat(&self, messages: Vec<ChatMsg>) -> Result<String, String> {
        let body = serde_json::json!({
            "model": self.cfg.model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content,
            })).collect::<Vec<_>>(),
            "temperature": 0.2,
        });
        let mut req = self.http.post(self.endpoint()).json(&body);
        if !self.cfg.api_key.is_empty() {
            req = req.bearer_auth(&self.cfg.api_key);
        }
        let resp = req.send().await.map_err(|e| format!("网络错误: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API 错误 {status}: {}", crate::core::config::truncate(&text, 300)));
        }
        let v: Value = resp.json().await.map_err(|e| format!("响应解析失败: {e}"))?;
        v["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "响应缺少 choices[0].message.content".to_string())
    }

    /// 结构化输出：要求回复 JSON（引导 + 提取 + 反序列化）
    pub async fn chat_json<T: DeserializeOwned>(&self, system: &str, user: &str) -> Result<T, String> {
        let content = self
            .chat(vec![
                ChatMsg { role: "system".into(), content: system.to_string() },
                ChatMsg { role: "user".into(), content: user.to_string() },
            ])
            .await?;
        let v = extract_json(&content).ok_or_else(|| {
            format!("未能从回复中提取 JSON: {}", crate::core::config::truncate(&content, 200))
        })?;
        serde_json::from_value(v).map_err(|e| format!("JSON 结构不符合要求: {e}"))
    }
}

/// 从回复文本提取第一个 JSON 值（支持 ```json 围栏）
pub fn extract_json(s: &str) -> Option<Value> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&s[start..=end]).ok()
}