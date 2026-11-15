//! What the model reads from the tools: only what it needs, in the conversation language. The full bilingual
//! package stays in the database for the interface and the sheet; the model never gets series it cannot cite.
use serde_json::{json, Map, Value};

/// A bilingual `{en, es}` value in one language (plain strings pass through).
fn l(v: &Value, lang: &str) -> Value {
    match v {
        Value::Object(o) if o.contains_key("en") => o.get(lang).or_else(|| o.get("en")).cloned().unwrap_or(Value::Null),
        other => other.clone(),
    }
}

fn put(o: &mut Map<String, Value>, k: &str, v: Value) { if !v.is_null() { o.insert(k.into(), v); } }

/// The stored evidence package, compacted: readings (no series, method or licence text), pests, a forecast
/// summary, the outlook, the projection and the gaps. About a fifth of the full package.
pub fn evidence(ev: &Value, lang: &str) -> Value {
    let readings: Vec<Value> = ev["readings"].as_array().into_iter().flatten().map(|r| {
        let mut o = Map::new();
        for k in ["id", "value", "unit", "source", "date", "badge"] { put(&mut o, k, r[k].clone()); }
        put(&mut o, "label", l(&r["label"], lang));
        put(&mut o, "verdict", l(&r["verdict"], lang));
        if r["provisional"] == true { o.insert("provisional".into(), json!(true)); }
        if r["note"].is_object() { put(&mut o, "note", l(&r["note"], lang)); }
        Value::Object(o)
    }).collect();
    let pests: Vec<Value> = ev["pests"].as_array().into_iter().flatten().map(|p| {
        let mut o = Map::new();
        put(&mut o, "name", l(&p["name"], lang));
        put(&mut o, "scientific", p["scientific"].clone());
        if p["crop"].is_object() { put(&mut o, "crop", l(&p["crop"]["name"], lang)); }
        put(&mut o, "role", p["role"].clone());
        put(&mut o, "generations_now", p["generations_now"].clone());
        put(&mut o, "generations_normal", p["generations_normal"].clone());
        if p["qualitative"] == true { o.insert("no_published_threshold".into(), json!(true)); }
        if p["reference"] == true { o.insert("general_reference_not_this_crop".into(), json!(true)); }
        Value::Object(o)
    }).collect();
    let forecast = ev["forecast"]["days"].as_array().filter(|d| !d.is_empty()).map(|days| {
        let f = |k: &str| days.iter().filter_map(|d| d[k].as_f64()).collect::<Vec<_>>();
        let hottest = days.iter().max_by(|a, b| a["tmax"].as_f64().unwrap_or(f64::MIN).partial_cmp(&b["tmax"].as_f64().unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal));
        json!({
            "source": ev["forecast"]["source"], "run": ev["forecast"]["run"], "badge": "regional context", "days": days.len(),
            "from": days.first().map(|d| d["date"].clone()), "to": days.last().map(|d| d["date"].clone()),
            "tmax_highest": hottest.map(|d| d["tmax"].clone()), "tmax_highest_date": hottest.map(|d| d["date"].clone()),
            "tmin_lowest": f("tmin").into_iter().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |m| m.min(x)))),
            "rain_total_mm": (f("rain_mm").iter().sum::<f64>() * 10.0).round() / 10.0,
            "days_tmax_29_or_more": f("tmax").iter().filter(|x| **x >= 29.0).count(),
        })
    });
    let outlook = ev["outlook"].is_object().then(|| json!({
        "label": l(&ev["outlook"]["label"], lang), "from": ev["outlook"]["from"], "to": ev["outlook"]["to"], "anomaly_c": ev["outlook"]["anomaly_c"],
        "months": ev["outlook"]["months"].as_array().into_iter().flatten().map(|m| json!({ "month": m["month"], "normal_mean": m["normal_mean"], "if_anomaly_persists": m["persist_mean"] })).collect::<Vec<_>>(),
        "pest_generations": ev["outlook"]["pests"].as_array().into_iter().flatten().map(|p| json!({ "name": l(&p["name"], lang), "so_far": p["to_date"], "by_end_normal": p["by_end_normal"], "by_end_if_warm": p["by_end_persist"] })).collect::<Vec<_>>(),
    }));
    let ndvi_last = ev["ndvi_series"].as_array().and_then(|s| s.last()).map(|n| json!({ "date": n["date"], "ndvi": n["ndvi"], "sensor": n["sensor"] }));
    let projection = ev["projection"].is_object().then(|| json!({
        "horizon": ev["projection"]["horizon"], "scenario": ev["projection"]["scenario"], "d_temp_c": ev["projection"]["d_temp_c"],
        "d_temp_range": ev["projection"]["d_temp_range"], "d_rain_pct": ev["projection"]["d_rain_pct"], "models": ev["projection"]["models"].as_array().map(|m| m.len()),
        "label": l(&ev["projection"]["label"], lang),
    }));
    json!({
        "language": lang,
        "field": { "label": ev["field"]["label"], "area_ha": ev["field"]["area_ha"] },
        "generated_at": ev["generated_at"],
        "readings": readings,
        "ndvi_latest": ndvi_last, "ndvi_points": ev["ndvi_series"].as_array().map(|s| s.len()),
        "bare_soil_periods": ev["bare_soil"].as_array().map(|b| b.iter().filter_map(|x| Some(format!("{} – {}", x["start"].as_str()?, x["end"].as_str()?))).collect::<Vec<_>>()),
        "pests": pests,
        "forecast": forecast,
        "outlook": outlook,
        "projection": projection,
        "imagery_date": ev["imagery"]["date"],
        "gaps": ev["gaps"].as_array().into_iter().flatten().map(|g| l(&g["text"], lang)).collect::<Vec<_>>(),
    })
}

/// The advice summary in one language: strategies (scores, why, best fit), item titles per section, gaps.
pub fn advice(a: &Value, lang: &str) -> Value {
    let list = |v: &Value| -> Vec<Value> { v.as_array().into_iter().flatten().map(|x| l(x, lang)).collect() };
    json!({
        "language": lang,
        "crop": { "name": l(&a["crop"]["name"], lang), "family": a["crop"]["family"], "crop_type": a["crop"]["crop_type"], "origin": a["crop"]["origin"] },
        "mode": a["mode"],
        "notice": l(&a["notice"], lang),
        "strategies": a["strategies"].as_array().into_iter().flatten().map(|s| json!({
            "name": l(&s["name"], lang), "current": s["current"], "scores_1_to_5": s["scores"], "why": l(&s["why"], lang), "best_for": l(&s["best_for"], lang),
        })).collect::<Vec<_>>(),
        "practices": list(&a["practices"]),
        "soil": list(&a["soil"]), "pest_care": list(&a["pest_care"]), "nutrition": list(&a["nutrition"]),
        "gaps": list(&a["gaps"]),
        "sections_written": a["sections_written"],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_drops_series_and_keeps_one_language() {
        let ev = json!({
            "field": { "label": "Huaral", "area_ha": 5.1 }, "generated_at": "2026-11-14",
            "readings": [{ "id": "temp_anomaly", "label": { "en": "Temperature", "es": "Temperatura" }, "verdict": { "en": "warmer", "es": "más cálido" },
                           "value": 5.0, "unit": "°C", "source": "NASA POWER", "date": "2026-11", "badge": "regional context",
                           "series": [{ "t": "2026-01", "v": 21.0 }], "method": "long text", "license": "NASA" }],
            "ndvi_series": [{ "date": "2026-11-13", "ndvi": 0.78, "sensor": "S2", "scene": "x" }],
            "pests": [{ "name": { "en": "Pea aphid", "es": "Pulgón de la arveja" }, "role": "current", "generations_now": 13.3, "generations_normal": 10.3, "crop": { "name": { "en": "pea", "es": "arveja" } } }],
            "gaps": [{ "id": "soil_lab", "text": { "en": "No soil lab", "es": "Sin análisis" } }],
        });
        let c = evidence(&ev, "es");
        let s = c.to_string();
        assert!(s.contains("Temperatura") && !s.contains("\"Temperature\""));
        assert!(!s.contains("series") || s.contains("ndvi_points"));
        assert!(!s.contains("long text") && !s.contains("\"scene\""));
        assert_eq!(c["pests"][0]["name"], "Pulgón de la arveja");
        assert_eq!(c["gaps"][0], "Sin análisis");
    }
}
