//! Token usage of every Anthropic call: one tracing line (target `usage`) and one row in `api_usage`. A failed
//! insert is logged and ignored, so measuring never breaks a conversation.
use serde_json::Value;
use sqlx::MySqlPool;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage { pub input: u64, pub output: u64, pub cache_read: u64, pub cache_write: u64 }

impl Usage {
    /// From a Messages API response's `usage` object.
    pub fn of(resp: &Value) -> Self {
        let u = &resp["usage"];
        let g = |k: &str| u[k].as_u64().unwrap_or(0);
        Usage { input: g("input_tokens"), output: g("output_tokens"), cache_read: g("cache_read_input_tokens"), cache_write: g("cache_creation_input_tokens") }
    }
    pub fn add(&mut self, o: Usage) { self.input += o.input; self.output += o.output; self.cache_read += o.cache_read; self.cache_write += o.cache_write; }
    /// Every input token the request carried (uncached + read from cache + written to cache).
    pub fn total_input(&self) -> u64 { self.input + self.cache_read + self.cache_write }
}

pub struct Call<'a> { pub analysis_id: Option<&'a str>, pub kind: &'a str, pub turn_key: Option<&'a str>, pub round: u32, pub model: &'a str }

pub async fn record(db: &MySqlPool, c: &Call<'_>, u: Usage) {
    tracing::info!(target: "usage", kind = c.kind, analysis = c.analysis_id.unwrap_or("-"), turn = c.turn_key.unwrap_or("-"), round = c.round,
        input = u.input, output = u.output, cache_read = u.cache_read, cache_write = u.cache_write, "anthropic usage");
    let r = sqlx::query("INSERT INTO api_usage (analysis_id, kind, turn_key, round, model, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(c.analysis_id).bind(c.kind).bind(c.turn_key).bind(c.round).bind(c.model)
        .bind(u.input).bind(u.output).bind(u.cache_read).bind(u.cache_write)
        .execute(db).await;
    if let Err(e) = r { tracing::warn!("usage not stored: {e}"); }
}

/// One line per finished turn with its totals.
pub fn log_turn(analysis_id: &str, turn_key: &str, rounds: u32, t: Usage) {
    tracing::info!(target: "usage", analysis = analysis_id, turn = turn_key, rounds, input = t.input, output = t.output,
        cache_read = t.cache_read, cache_write = t.cache_write, total_input = t.total_input(), "turn usage");
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_usage() {
        let u = super::Usage::of(&serde_json::json!({ "usage": { "input_tokens": 10, "output_tokens": 5, "cache_read_input_tokens": 100, "cache_creation_input_tokens": 20 } }));
        assert_eq!((u.input, u.output, u.cache_read, u.cache_write, u.total_input()), (10, 5, 100, 20, 130));
    }
}
