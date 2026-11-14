//! Voice dictation: the farmer's recording → text with Gemini (audio understanding). Only transcribes; never sends a
//! chat message. The key comes from env.txt (`gemini`) and is sent in a header, never logged.
use base64::Engine;
use common::AppState;
use serde_json::{json, Value};

const MODEL: &str = "gemini-3.8-flash";
pub const MAX_AUDIO_BYTES: usize = 10 * 1024 * 1024;

pub async fn transcribe(state: &AppState, mime: &str, audio: &[u8], lang: &str) -> anyhow::Result<String> {
    let key = state.config.secrets.gemini.as_deref().ok_or_else(|| anyhow::anyhow!("transcription is not configured"))?;
    anyhow::ensure!(!audio.is_empty() && audio.len() <= MAX_AUDIO_BYTES, "audio must be between 1 byte and 10 MB");
    let mime = mime.split(';').next().unwrap_or("audio/webm").trim();
    let hint = if lang == "es" { "The speaker most likely talks in Spanish (Peru)" } else { "The speaker most likely talks in English" };
    let body = json!({
        "contents": [{ "parts": [
            { "text": format!("Transcribe this recording verbatim, in the language actually spoken. {hint}, but keep any other language as spoken. Farm vocabulary is expected (crop, pest and fertilizer names). Return only the transcript text, with normal punctuation; no notes, no quotes, no translation. If nothing intelligible is said, return an empty string.") },
            { "inline_data": { "mime_type": mime, "data": base64::engine::general_purpose::STANDARD.encode(audio) } }
        ]}],
        "generationConfig": { "temperature": 0 }
    });
    let resp = state.http
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent"))
        .header("x-goog-api-key", key)
        .timeout(std::time::Duration::from_secs(90))
        .json(&body).send().await
        .map_err(|e| anyhow::anyhow!("transcription request failed: {}", e.without_url()))?;
    let status = resp.status();
    let v: Value = resp.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        anyhow::bail!("transcription failed ({}): {}", status.as_u16(), v.pointer("/error/message").and_then(Value::as_str).unwrap_or("unknown error"));
    }
    let text = v.pointer("/candidates/0/content/parts").and_then(Value::as_array)
        .map(|parts| parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    Ok(text.trim().trim_matches('"').to_string())
}
