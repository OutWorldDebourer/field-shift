//! One conversation turn: store the farmer's message and photos, run the live consultant (tool loop) or the demo
//! script, store the reply and return the new messages and sections.
use crate::{anthropic, demo, prompt, ratelimit, tools};
use analyses::{photos, store, AnalysisState, ChatMessage, PhotoRef, Section};
use common::{usage::{self, Usage}, ApiError, ApiResult, AppState};
use serde_json::{json, Value};

pub struct NewPhoto {
    pub mime: String,
    pub bytes: Vec<u8>,
    /// Original file name (shown for documents).
    pub name: Option<String>,
}

pub struct TurnInput {
    pub text: String,
    pub lang: String,
    pub photos: Vec<NewPhoto>,
    pub visitor: String,
    pub force_demo: bool,
    /// The tab the farmer is on when sending (chat, dashboard, rotation, organic, prevention, sheet).
    pub view: Option<String>,
    /// The card they focused or asked about, e.g. "rotation · strategy 2 (Follow with maize)".
    pub focus: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TurnOutput {
    pub messages: Vec<ChatMessage>,
    pub sections: Vec<Section>,
    pub demo_mode: bool,
    pub steps: Vec<analyses::steps::Step>,
    pub next_step: Option<String>,
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
    let mut refs: Vec<PhotoRef> = saved.iter().zip(&input.photos).map(|(p, n)| PhotoRef {
        url: p.url.clone(), tag: None,
        kind: Some(if p.mime == "application/pdf" { "pdf".into() } else { "image".into() }),
        name: n.name.clone(),
    }).collect();
    let user_msg = store::add_message(&state.db, id, "user", input.text.trim(), &refs).await?;
    photos::set_message(state, &saved.iter().map(|p| p.id).collect::<Vec<_>>(), user_msg).await?;

    let key = state.config.secrets.anthropic.clone();
    let live_ok = !input.force_demo && key.is_some() && ratelimit::allow(&state.db, &input.visitor).await.unwrap_or(false);
    let (reply, demo_mode, tags) = if live_ok {
        match live(state, id, &before, &input, &saved, key.as_deref().unwrap_or_default(), &user_msg.to_string()).await {
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
        // Photo numbers count images only (documents are numbered separately).
        if let Some(r) = refs.iter_mut().filter(|r| r.kind.as_deref() != Some("pdf")).nth(i.saturating_sub(1)) { r.tag = Some(tag); }
    }
    if refs.iter().any(|r| r.tag.is_some()) {
        sqlx::query("UPDATE messages SET meta = ? WHERE id = ?").bind(json!({ "photos": refs }).to_string()).bind(user_msg).execute(&state.db).await?;
    }
    store::add_message(&state.db, id, "assistant", &reply, &[]).await?;
    store::set_demo_mode(&state.db, id, demo_mode).await?;
    let after = store::load(&state.db, id).await?;
    Ok(TurnOutput { messages: after.messages.into_iter().filter(|m| m.id >= user_msg).collect(), sections: after.sections, demo_mode, steps: after.steps, next_step: after.next_step })
}

/// The consultant's opening message, written by the model from the real place, coordinates and area of the field.
/// Idempotent: if the conversation already has a consultant message, nothing is added. Without the API (no key,
/// rate limit, demo, failure) the template greeting is used, with the same real place name.
pub async fn run_start(state: &AppState, id: &str, lang: &str, visitor: &str, force_demo: bool) -> ApiResult<TurnOutput> {
    let st = store::load(&state.db, id).await?;
    if !st.messages.iter().any(|m| m.role == "assistant") {
        store::set_lang(&state.db, id, lang).await?;
        let key = state.config.secrets.anthropic.clone();
        let live_ok = !force_demo && key.is_some() && ratelimit::allow(&state.db, visitor).await.unwrap_or(false);
        let text = if live_ok { opening(state, &st, lang, key.as_deref().unwrap_or_default()).await.map_err(|e| tracing::warn!("opening failed: {e}")).ok() } else { None };
        match text {
            Some(t) => { store::add_message(&state.db, id, "assistant", &t, &[]).await?; store::set_demo_mode(&state.db, id, false).await?; }
            None => store::add_greeting(&state.db, id, lang).await?,
        }
    }
    let after = store::load(&state.db, id).await?;
    Ok(TurnOutput { messages: after.messages, sections: after.sections, demo_mode: after.demo_mode, steps: after.steps, next_step: after.next_step })
}

async fn opening(state: &AppState, st: &AnalysisState, lang: &str, key: &str) -> anyhow::Result<String> {
    let client = anthropic::Client { http: &state.http, key };
    let ask = format!("{}

[Opening — not written by the farmer] The farmer has just marked this field on the map and opened the chat. Write your first message in the interface language ({lang}). Greet warmly in one short line. Say where the field is: the place name and the coordinates from the context note (as degrees with N/S and E/W, two decimals), and the area if it is above 0. In one sentence, say what you will read for this exact plot from NASA and satellites: heat against a 40-year normal, rain, soil moisture, crop vigor and the 10-day forecast. Say you will cover five topics — **your field**, **soil**, **pest care**, **nutrition** and **rotation** — ending with a technical sheet for their agronomist, and that the \"Load information\" form answers them all in one screen. If you are confident about what is typically farmed around that place, you may mention it briefly as a question (never as a fact about this plot). End with one short checklist (numbered, a few words per line, \"I don't know\" is fine) of the first things you need: crop now, crops before, irrigation, insects or damage seen. No figures other than the coordinates and area. Under 110 words. Bold the place name.", prompt::context_note(st, lang));
    let body = json!({
        "model": anthropic::MODEL,
        "max_tokens": 2000,
        "system": [{ "type": "text", "text": prompt::SYSTEM, "cache_control": { "type": "ephemeral" } }],
        "output_config": { "effort": "low" },
        "messages": [{ "role": "user", "content": ask }],
    });
    let resp = client.create(&body).await?;
    usage::record(&state.db, &usage::Call { analysis_id: Some(&st.id), kind: "opening", turn_key: Some("opening"), round: 1, model: anthropic::MODEL }, Usage::of(&resp)).await;
    if resp["stop_reason"] == "refusal" { anyhow::bail!("refusal"); }
    let t = anthropic::text_of(&resp);
    if t.trim().is_empty() { anyhow::bail!("empty opening"); }
    Ok(t)
}

async fn live(state: &AppState, id: &str, st: &AnalysisState, input: &TurnInput, saved: &[photos::StoredPhoto], key: &str, turn_key: &str)
    -> anyhow::Result<(String, tools::Effects)> {
    let client = anthropic::Client { http: &state.http, key };
    let mut messages = history(st);
    // Each photo in full, then (for the first four large ones) its quarters at native resolution for tiny insects.
    let mut content: Vec<Value> = vec![];
    let (mut n_photo, mut n_doc) = (0, 0);
    for (i, p) in saved.iter().enumerate() {
        if p.mime == "application/pdf" {
            // Documents (an earlier technical sheet, a lab report…) go to Claude as PDF document blocks.
            n_doc += 1;
            let name = input.photos.get(i).and_then(|x| x.name.clone()).unwrap_or_else(|| format!("document {n_doc}"));
            content.push(json!({ "type": "text", "text": format!("Document {n_doc} ({name}):") }));
            content.push(json!({ "type": "document", "source": { "type": "base64", "media_type": "application/pdf",
                "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &p.bytes) }, "title": name }));
            continue;
        }
        n_photo += 1;
        let i = n_photo - 1;
        content.push(json!({ "type": "text", "text": format!("Photo {}:", i + 1) }));
        let (media, data) = crate::tiles::model_view(&p.bytes, &p.mime);
        content.push(json!({ "type": "image", "source": { "type": "base64", "media_type": media, "data": data } }));
        if i < 4 {
            for tile in crate::tiles::quarters(&p.bytes) {
                content.push(json!({ "type": "text", "text": format!("Photo {}, {} at full resolution (for small insects):", i + 1, tile.label) }));
                content.push(json!({ "type": "image", "source": { "type": "base64", "media_type": "image/jpeg", "data": tile.jpeg_b64 } }));
            }
        }
    }
    content.push(json!({ "type": "text", "text": prompt::context_note(st, &input.lang) }));
    if let Some(v) = prompt::viewing_note(input.view.as_deref(), input.focus.as_deref()) { content.push(json!({ "type": "text", "text": v })); }
    let text = if !input.text.trim().is_empty() { input.text.trim().to_string() } else if n_doc > 0 { "(documents attached)".to_string() } else { "(photos only)".to_string() };
    content.push(json!({ "type": "text", "text": text }));
    push(&mut messages, "user", Value::Array(content));

    let mut fx = tools::Effects::default();
    let mut total = Usage::default();
    let mut rounds = 0u32;
    let out: anyhow::Result<String> = async {
    for round in 1..=MAX_TOOL_ROUNDS as u32 {
        let body = round_body(&messages);
        let resp = client.create(&body).await?;
        let u = Usage::of(&resp);
        usage::record(&state.db, &usage::Call { analysis_id: Some(id), kind: "turn", turn_key: Some(turn_key), round, model: anthropic::MODEL }, u).await;
        total.add(u);
        rounds = round;
        match resp["stop_reason"].as_str() {
            Some("tool_use") => {
                let blocks = resp["content"].clone();
                let mut results = vec![];
                for b in blocks.as_array().into_iter().flatten().filter(|b| b["type"] == "tool_use") {
                    let name = b["name"].as_str().unwrap_or("");
                    let out = tools::execute(state, id, name, &b["input"], &mut fx).await;
                    let (txt, err) = match out { Ok(v) => (tools::result_text(&v), false), Err(e) => (e, true) };
                    results.push(json!({ "type": "tool_result", "tool_use_id": b["id"], "content": txt, "is_error": err }));
                }
                messages.push(json!({ "role": "assistant", "content": blocks }));
                messages.push(json!({ "role": "user", "content": results }));
            }
            Some("refusal") => {
                let t = if input.lang == "es" { "No puedo ayudar con eso. Sigamos con tu campo: ¿qué cultivas ahora?" } else { "I can't help with that. Let's get back to your field: what are you growing now?" };
                return Ok(t.to_string());
            }
            _ => {
                let t = anthropic::text_of(&resp);
                if t.is_empty() { anyhow::bail!("empty reply"); }
                return Ok(t);
            }
        }
    }
    anyhow::bail!("tool loop did not finish")
    }.await;
    usage::log_turn(id, turn_key, rounds, total);
    out.map(|t| (t, fx))
}

/// One request of the tool loop: stable system prompt and tools first (cached), then the conversation.
/// The last block of the last message carries a cache breakpoint, so every later round of the tool loop reads the
/// whole earlier conversation from the cache (the system prompt keeps its own breakpoint).
pub(crate) fn round_body(messages: &[Value]) -> Value {
    let mut messages = messages.to_vec();
    if let Some(last) = messages.last_mut() {
        if let Value::String(s) = &last["content"] { last["content"] = json!([{ "type": "text", "text": s }]); }
        if let Some(b) = last["content"].as_array_mut().and_then(|a| a.last_mut()) { b["cache_control"] = json!({ "type": "ephemeral" }); }
    }
    json!({
        "model": anthropic::MODEL,
        "max_tokens": 16000,
        "system": [{ "type": "text", "text": prompt::SYSTEM, "cache_control": { "type": "ephemeral" } }],
        "tools": tools::definitions(),
        "output_config": { "effort": "medium" },
        "messages": messages,
    })
}

/// The messages of a text-only turn: this analysis's history, then the context note and the farmer's text.
pub(crate) fn text_turn(st: &AnalysisState, text: &str, lang: &str) -> Vec<Value> {
    let mut messages = history(st);
    let content = vec![json!({ "type": "text", "text": prompt::context_note(st, lang) }), json!({ "type": "text", "text": text })];
    push(&mut messages, "user", Value::Array(content));
    messages
}

/// Earlier turns as plain text (photos summarised), alternating roles, starting with a user turn.
pub(crate) fn history(st: &AnalysisState) -> Vec<Value> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use analyses::FieldOut;

    fn state(id: &str, label: &str, said: &[(&str, &str)]) -> AnalysisState {
        serde_json::from_value(json!({
            "id": id, "lang": "en", "location_mode": "map", "example_site": null,
            "field": FieldOut { center: common::geo::LatLon { lat: -11.47, lon: -77.2 }, ring: vec![], area_ha: 2.0, label: label.into() },
            "crop_current": null, "crop_history": [], "sections": [], "evidence": null, "demo_mode": false, "prediagnosis": null,
            "steps": [], "facts": null, "next_step": null,
            "messages": said.iter().enumerate().map(|(i, (role, body))| json!({ "id": i, "role": role, "body": body, "created_at": "2026-11-14" })).collect::<Vec<_>>(),
        })).expect("fixture")
    }

    /// A turn's request carries only this analysis's messages, its context note, the farmer's new text and the tool
    /// results of this turn: nothing from another analysis, even one by the same visitor.
    #[test]
    fn a_turn_sees_only_its_own_analysis() {
        let a = state("a", "Plot ALPHA", &[("assistant", "Hello ALPHA-GREETING"), ("user", "ALPHA-CROP peas"), ("assistant", "ALPHA-REPLY noted")]);
        let _b = state("b", "Plot BRAVO", &[("assistant", "Hello BRAVO-GREETING"), ("user", "BRAVO-CROP grapes")]);
        let mut messages = text_turn(&a, "ALPHA-NEW what about water?", "en");
        messages.push(json!({ "role": "assistant", "content": [{ "type": "tool_use", "id": "t1", "name": "get_evidence", "input": {} }] }));
        messages.push(json!({ "role": "user", "content": [{ "type": "tool_result", "tool_use_id": "t1", "content": "ALPHA-EVIDENCE" }] }));
        let body = round_body(&messages).to_string();
        for want in ["ALPHA-GREETING", "ALPHA-CROP", "ALPHA-REPLY", "ALPHA-NEW", "Plot ALPHA", "ALPHA-EVIDENCE", "[Context for the consultant"] {
            assert!(body.contains(want), "missing {want}");
        }
        assert!(!body.contains("BRAVO"), "another analysis leaked into the request");
        // Exactly: history ("Hello" opener + 3 messages) + this turn + one tool round.
        let msgs = round_body(&messages)["messages"].as_array().unwrap().len();
        assert_eq!(msgs, 7);
        // The conversation tail carries the cache breakpoint; earlier blocks do not.
        let v = round_body(&messages);
        assert!(v["messages"][6]["content"][0]["cache_control"].is_object());
        assert!(v["messages"][4]["content"].to_string().matches("cache_control").count() == 0);
    }
}
