//! Minimal Anthropic Messages API client over reqwest (Rust has no official SDK). The key is sent in a header and
//! never logged; errors carry only the status and the API's error message.
use serde_json::{json, Value};

pub const MODEL: &str = "claude-opus-5-5";
const URL: &str = "https://api.anthropic.com/v1/messages";
const VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

pub struct Client<'a> {
    pub http: &'a reqwest::Client,
    pub key: &'a str,
}

impl Client<'_> {
    /// One Messages request. Tries with server-side refusal fallbacks first; if the API rejects that option, retries without.
    pub async fn create(&self, body: &Value) -> anyhow::Result<Value> {
        let mut with_fb = body.clone();
        with_fb["fallbacks"] = json!("default");
        match self.post(&with_fb, true).await {
            Ok(v) => Ok(v),
            Err(e) if e.to_string().contains("400") => self.post(body, false).await,
            Err(e) => Err(e),
        }
    }

    async fn post(&self, body: &Value, beta: bool) -> anyhow::Result<Value> {
        let mut req = self.http.post(URL)
            .header("x-api-key", self.key)
            .header("anthropic-version", VERSION)
            .header("content-type", "application/json")
            .timeout(std::time::Duration::from_secs(180))
            .json(body);
        if beta { req = req.header("anthropic-beta", FALLBACK_BETA); }
        let resp = req.send().await.map_err(|e| anyhow::anyhow!("anthropic request failed: {}", e.without_url()))?;
        let status = resp.status();
        let v: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            let msg = v.pointer("/error/message").and_then(Value::as_str).unwrap_or("unknown error");
            anyhow::bail!("anthropic {}: {}", status.as_u16(), msg);
        }
        Ok(v)
    }
}

/// Concatenated text blocks of a response.
pub fn text_of(resp: &Value) -> String {
    resp["content"].as_array().map(|blocks| {
        blocks.iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join("\n")
    }).unwrap_or_default().trim().to_string()
}
