//! Tiny HTML helpers (escaping, bilingual picking).
use serde_json::Value;

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/// Picks `lang` from a `{en, es}` value (falls back to English, then to a plain string).
pub fn bi(v: &Value, lang: &str) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get(lang).or_else(|| o.get("en")).and_then(Value::as_str).unwrap_or("").to_string(),
        _ => String::new(),
    }
}

pub fn num(v: &Value) -> String {
    match v.as_f64() {
        Some(x) if x.abs() >= 100.0 => format!("{x:.0}"),
        Some(x) => format!("{}", (x * 100.0).round() / 100.0),
        None => "—".into(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn escapes() { assert_eq!(super::esc("<a href=\"x\">&"), "&lt;a href=&quot;x&quot;&gt;&amp;"); }
}
