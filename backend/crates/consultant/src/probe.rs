//! Token probe (development tool, no paid calls): replays one typical consultant turn on an analysis with
//! scripted tool calls (set_crops → get_evidence + get_advice → complete_step), runs the real tools, and counts the
//! input tokens of every tool-loop request with the free `count_tokens` endpoint. Used to compare request size
//! before and after a change (BUILD_PLAN decision log).
use crate::{tools, turn};
use common::AppState;
use serde_json::{json, Value};

pub struct Round { pub label: String, pub input_tokens: u64, pub bytes: usize }
pub struct ToolSize { pub name: String, pub chars: usize }

async fn count(state: &AppState, key: &str, body: &Value) -> anyhow::Result<u64> {
    let b = json!({ "model": body["model"], "system": body["system"], "tools": body["tools"], "messages": body["messages"] });
    let resp = state.http.post("https://api.anthropic.com/v1/messages/count_tokens")
        .header("x-api-key", key).header("anthropic-version", "2026-10-21").json(&b).send().await
        .map_err(|e| anyhow::anyhow!("count_tokens request failed: {}", e.without_url()))?;
    let status = resp.status();
    let v: Value = resp.json().await.unwrap_or(Value::Null);
    if !status.is_success() { anyhow::bail!("count_tokens {}: {}", status.as_u16(), v.pointer("/error/message").and_then(Value::as_str).unwrap_or("")); }
    Ok(v["input_tokens"].as_u64().unwrap_or(0))
}

/// Tokens of the always-cached prefix (tool definitions + system prompt), with a one-word message.
pub async fn prefix_tokens(state: &AppState) -> anyhow::Result<u64> {
    let key = state.config.secrets.anthropic.clone().ok_or_else(|| anyhow::anyhow!("no anthropic key"))?;
    count(state, &key, &turn::round_body(&[json!({ "role": "user", "content": "x" })])).await
}

/// Replays the scripted turn on `id` (its tools really run and write to that analysis).
pub async fn run(state: &AppState, id: &str, text: &str, lang: &str) -> anyhow::Result<(Vec<Round>, Vec<ToolSize>)> {
    let key = state.config.secrets.anthropic.clone().ok_or_else(|| anyhow::anyhow!("no anthropic key"))?;
    let st = analyses::store::load(&state.db, id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut messages = turn::text_turn(&st, text, lang);
    let script: Vec<Vec<(&str, Value)>> = vec![
        vec![("set_crops", json!({ "current": "pea", "previous": ["lettuce"] }))],
        vec![("get_evidence", json!({})), ("get_advice", json!({}))],
        vec![("complete_step", json!({ "key": "field", "summary_en": "Peas now, lettuce before; furrow irrigation.", "summary_es": "Arveja ahora, lechuga antes; riego por surcos." }))],
    ];
    let (mut rounds, mut sizes) = (vec![], vec![]);
    let mut fx = tools::Effects::default();
    for (i, calls) in script.iter().enumerate() {
        let body = turn::round_body(&messages);
        rounds.push(Round { label: format!("round {}", i + 1), input_tokens: count(state, &key, &body).await?, bytes: body.to_string().len() });
        let mut uses = vec![];
        let mut results = vec![];
        for (j, (name, input)) in calls.iter().enumerate() {
            let tid = format!("toolu_probe_{i}_{j}");
            uses.push(json!({ "type": "tool_use", "id": tid, "name": name, "input": input }));
            let out = tools::execute(state, id, name, input, &mut fx).await;
            let (txt, err) = match out { Ok(v) => (tools::result_text(&v), false), Err(e) => (e, true) };
            sizes.push(ToolSize { name: name.to_string(), chars: txt.chars().count() });
            results.push(json!({ "type": "tool_result", "tool_use_id": tid, "content": txt, "is_error": err }));
        }
        messages.push(json!({ "role": "assistant", "content": uses }));
        messages.push(json!({ "role": "user", "content": results }));
    }
    let body = turn::round_body(&messages);
    rounds.push(Round { label: format!("round {} (final answer)", script.len() + 1), input_tokens: count(state, &key, &body).await?, bytes: body.to_string().len() });
    Ok((rounds, sizes))
}
