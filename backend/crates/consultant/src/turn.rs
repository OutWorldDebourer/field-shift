//! One conversation turn: store the farmer's message and photos, run the live consultant (tool loop) or the demo
//! script, store the reply and return the new messages and sections.
use crate::{anthropic, demo, prompt, ratelimit, tools};
use analyses::{photos, store, AnalysisState, ChatMessage, PhotoRef, Section};
use base64::Engine;
use common::{ApiError, ApiResult, AppState};
use serde_json::{json, Value};

pub struct NewPhoto {
    pub mime: String,
    pub bytes: Vec<u8>,
}

pub struct TurnInput {
    pub text: String,
    pub lang: String,
    pub photos: Vec<NewPhoto>,
    pub visitor: String,
    pub force_demo: bool,
}

#[derive(serde::Serialize)]
pub struct TurnOutput {
    pub messages: Vec<ChatMessage>,
    pub sections: Vec<Section>,
    pub demo_mode: bool,
}

const MAX_TOOL_ROUNDS: usize = 8;

pub async fn run_turn(state: &AppState, id: &str, input: TurnInput) -> ApiResult<TurnOutput> {
    if input.text.trim().is_empty() && input.photos.is_empty() {
        return Err(ApiError::BadRequest("empty message".into()));
    }
    if input.photos.len() > photos::MAX_PHOTOS_PER_MESSAGE {
        return Err(ApiError::BadRequest("up to 10 photos per message".into()));
    }
    if input.text.len() > 4000 { return Err(ApiError::BadRequest("message too long".into())); }
    let before = store::load(&state.db, id).await?;
    store::set_lang(&state.db, id, &input.lang).await?;

    let mut saved = vec![];
    for p in &input.photos { saved.push(photos::save(state, id, &p.mime, p.bytes.clone()).await?); }
    let mut refs: Vec<PhotoRef> = saved.iter().map(|p| PhotoRef { url: p.url.clone(), tag: None }).collect();
    let user_msg = store::add_message(&state.db, id, "user", input.text.trim(), &refs).await?;
    photos::set_message(state, &saved.iter().map(|p| p.id).collect::<Vec<_>>(), user_msg).await?;

    let key = state.config.secrets.anthropic.clone();
    let live_ok = !input.force_demo && key.is_some() && ratelimit::allow(&state.db, &input.visitor).await.unwrap_or(false);
    let (reply, demo_mode, tags) = if live_ok {
        match live(state, id, &before, &input, &saved, key.as_deref().unwrap_or_default()).await {
            Ok((text, fx)) => (text, false, fx.photo_tags),
            Err(e) => {
                tracing::warn!("live consultant failed, falling back to demo: {e}");
                (demo::reply(state, &before, &input).await?, true, vec![])
            }
        }
    } else {
        (demo::reply(state, &before, &input).await?, true, vec![])
    };
    for (i, tag) in tags {
        if let Some(r) = refs.get_mut(i.saturating_sub(1)) { r.tag = Some(tag); }
    }
    if refs.iter().any(|r| r.tag.is_some()) {
        sqlx::query("UPDATE messages SET meta = ? WHERE id = ?").bind(json!({ "photos": refs })).bind(user_msg).execute(&state.db).await?;
    }
    store::add_message(&state.db, id, "assistant", &reply, &[]).await?;
    store::set_demo_mode(&state.db, id, demo_mode).await?;
    let after = store::load(&state.db, id).await?;
    Ok(TurnOutput { messages: after.messages.into_iter().filter(|m| m.id >= user_msg).collect(), sections: after.sections, demo_mode })
}

async fn live(state: &AppState, id: &str, st: &AnalysisState, input: &TurnInput, saved: &[photos::StoredPhoto], key: &str)
    -> anyhow::Result<(String, tools::Effects)> {
    let client = anthropic::Client { http: &state.http, key };
    let mut messages = history(st);
    let mut content: Vec<Value> = saved.iter().enumerate().flat_map(|(i, p)| [
        json!({ "type": "text", "text": format!("Photo {}:", i + 1) }),
        json!({ "type": "image", "source": { "type": "base64", "media_type": p.mime, "data": base64::engine::general_purpose::STANDARD.encode(&p.bytes) } }),
    ]).collect();
    content.push(json!({ "type": "text", "text": prompt::context_note(st, &input.lang) }));
    let text = if input.text.trim().is_empty() { "(photos only)".to_string() } else { input.text.trim().to_string() };
    content.push(json!({ "type": "text", "text": text }));
    push(&mut messages, "user", Value::Array(content));

    let mut fx = tools::Effects::default();
    for _ in 0..MAX_TOOL_ROUNDS {
        let body = json!({
            "model": anthropic::MODEL,
            "max_tokens": 16000,
            "system": [{ "type": "text", "text": prompt::SYSTEM, "cache_control": { "type": "ephemeral" } }],
            "tools": tools::definitions(),
            "output_config": { "effort": "medium" },
            "messages": messages,
        });
        let resp = client.create(&body).await?;
        match resp["stop_reason"].as_str() {
            Some("tool_use") => {
                let blocks = resp["content"].clone();
                let mut results = vec![];
                for b in blocks.as_array().into_iter().flatten().filter(|b| b["type"] == "tool_use") {
                    let name = b["name"].as_str().unwrap_or("");
                    let out = tools::execute(state, id, name, &b["input"], &mut fx).await;
                    let (txt, err) = match out { Ok(v) => (v.to_string(), false), Err(e) => (e, true) };
                    results.push(json!({ "type": "tool_result", "tool_use_id": b["id"], "content": txt, "is_error": err }));
                }
                messages.push(json!({ "role": "assistant", "content": blocks }));
                messages.push(json!({ "role": "user", "content": results }));
            }
            Some("refusal") => {
                let t = if input.lang == "es" { "No puedo ayudar con eso. Sigamos con tu campo: ¿qué cultivas ahora?" } else { "I can't help with that. Let's get back to your field: what are you growing now?" };
                return Ok((t.to_string(), fx));
            }
            _ => {
                let t = anthropic::text_of(&resp);
                if t.is_empty() { anyhow::bail!("empty reply"); }
                return Ok((t, fx));
            }
        }
    }
    anyhow::bail!("tool loop did not finish")
}

/// Earlier turns as plain text (photos summarised), alternating roles, starting with a user turn.
fn history(st: &AnalysisState) -> Vec<Value> {
    let mut out: Vec<Value> = vec![];
    for m in st.messages.iter().filter(|m| m.role == "user" || m.role == "assistant") {
        let mut text = m.body.clone();
        if !m.photos.is_empty() { text = format!("[{} photo(s) sent earlier] {text}", m.photos.len()); }
        if text.trim().is_empty() { continue; }
        if out.is_empty() && m.role == "assistant" { push(&mut out, "user", json!("Hello")); }
        push(&mut out, &m.role, json!(text));
    }
    if let Some(last) = out.last() { if last["role"] == "user" { push(&mut out, "assistant", json!("(no reply)")); } }
    out
}

fn push(out: &mut Vec<Value>, role: &str, content: Value) {
    if let Some(last) = out.last_mut() {
        if last["role"] == role {
            let as_blocks = |c: &Value| match c { Value::String(s) => vec![json!({ "type": "text", "text": s })], Value::Array(a) => a.clone(), _ => vec![] };
            let mut merged = as_blocks(&last["content"]);
            merged.extend(as_blocks(&content));
            last["content"] = Value::Array(merged);
            return;
        }
    }
    out.push(json!({ "role": role, "content": content }));
}
