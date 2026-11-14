//! CSV of the evidence: one row per reading, then every series (reading series, NDVI, monthly climate).
use serde_json::Value;

fn cell(s: &str) -> String {
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

fn val(v: &Value) -> String {
    match v { Value::Null => String::new(), Value::String(s) => s.clone(), other => other.to_string() }
}

pub fn render(ev: &Value) -> String {
    let mut out = String::from("# Field Shift evidence export. Every value lists its source, date, resolution and licence.\n");
    out.push_str("table,id,label,date_or_period,value,unit,source,dataset,resolution,badge,license\n");
    for r in ev["readings"].as_array().into_iter().flatten() {
        let row = [
            "reading".to_string(), val(&r["id"]), val(&r["label"]["en"]), val(&r["date"]), val(&r["value"]), val(&r["unit"]),
            val(&r["source"]), val(&r["dataset"]), val(&r["resolution"]), val(&r["badge"]), val(&r["license"]),
        ];
        out.push_str(&row.iter().map(|c| cell(c)).collect::<Vec<_>>().join(","));
        out.push('\n');
        for p in r["series"].as_array().into_iter().flatten() {
            let row = ["series".to_string(), val(&r["id"]), val(&r["label"]["en"]), val(&p["t"]), val(&p["v"]), val(&r["unit"]),
                val(&r["source"]), val(&r["dataset"]), val(&r["resolution"]), val(&r["badge"]), val(&r["license"])];
            out.push_str(&row.iter().map(|c| cell(c)).collect::<Vec<_>>().join(","));
            out.push('\n');
        }
    }
    for p in ev["ndvi_series"].as_array().into_iter().flatten() {
        let row = ["ndvi".to_string(), val(&p["scene"]), "NDVI field mean".into(), val(&p["date"]), val(&p["ndvi"]), "".into(),
            val(&p["sensor"]), val(&p["scene"]), "".into(), "field-level".into(), "".into()];
        out.push_str(&row.iter().map(|c| cell(c)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    for m in ev["climate_monthly"].as_array().into_iter().flatten() {
        for (k, unit) in [("t_mean", "°C"), ("normal_mean", "°C"), ("normal_lo", "°C"), ("normal_hi", "°C"), ("rain", "mm"), ("normal_rain", "mm")] {
            let row = ["climate_monthly".to_string(), k.into(), k.into(), val(&m["month"]), val(&m[k]), unit.into(),
                "NASA POWER".into(), "POWER daily, AG community".into(), "~0.5° x 0.625°".into(), "regional context".into(), "NASA open data".into()];
            out.push_str(&row.iter().map(|c| cell(c)).collect::<Vec<_>>().join(","));
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn quotes_commas() {
        let ev = serde_json::json!({ "readings": [{ "id": "a", "label": { "en": "x, y" }, "value": 1.5 }] });
        assert!(super::render(&ev).contains("\"x, y\""));
    }
}
