//! The Reading shape of `frontend/src/shared/contract.ts`, plus helpers for plain-words verdicts.
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize)]
pub struct Bi {
    pub en: String,
    pub es: String,
}

pub fn bi(en: impl Into<String>, es: impl Into<String>) -> Bi { Bi { en: en.into(), es: es.into() } }

#[derive(Clone, Debug, Serialize)]
pub struct Reading {
    pub id: String,
    pub label: Bi,
    pub value: Option<f64>,
    pub unit: String,
    pub verdict: Bi,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub key: bool,
    pub source: String,
    pub dataset: String,
    pub date: String,
    pub resolution: String,
    pub badge: String,
    pub license: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<Value>,
    /// True when the latest values come from near-real-time streams (shown as "provisional").
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub provisional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Bi>,
}

pub fn gap(id: &str, en: impl Into<String>, es: impl Into<String>) -> Value {
    json!({ "id": id, "text": bi(en, es) })
}

pub fn round(x: f64, d: i32) -> f64 { let p = 10f64.powi(d); (x * p).round() / p }

/// "+1.2" style signed number.
pub fn signed(x: f64, d: usize) -> String { format!("{}{:.*}", if x >= 0.0 { "+" } else { "" }, d, x) }

/// Temperature anomaly words.
pub fn temp_verdict(anom: f64) -> Bi {
    let a = signed(anom, 1);
    if anom >= 2.0 { bi(format!("much warmer than normal ({a} °C)"), format!("mucho más cálido que lo normal ({a} °C)")) }
    else if anom >= 0.5 { bi(format!("warmer than normal ({a} °C)"), format!("más cálido que lo normal ({a} °C)")) }
    else if anom <= -2.0 { bi(format!("much cooler than normal ({a} °C)"), format!("mucho más frío que lo normal ({a} °C)")) }
    else if anom <= -0.5 { bi(format!("cooler than normal ({a} °C)"), format!("más frío que lo normal ({a} °C)")) }
    else { bi("close to normal", "cerca de lo normal") }
}

/// Crop vigor words for an NDVI field mean.
pub fn ndvi_verdict(v: f64) -> Bi {
    if v < 0.2 { bi("bare or very sparse cover", "suelo desnudo o cobertura muy escasa") }
    else if v < 0.4 { bi("low vigor", "vigor bajo") }
    else if v < 0.6 { bi("moderate vigor", "vigor moderado") }
    else { bi("high vigor", "vigor alto") }
}
