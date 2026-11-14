//! NASA POWER summary and monthly series → readings, pest table and the "this season vs normal" chart rows.
use crate::reading::{bi, round, signed, temp_verdict, Reading};
use serde_json::{json, Value};

fn metric<'a>(s: &'a Value, period: &str, key: &str) -> Option<&'a Value> {
    s["periods"].as_array()?.iter().find(|p| p["name"] == period)?["metrics"].as_array()?.iter().find(|m| m["key"] == key)
}

fn base(m: &Value, id: &str, label: crate::reading::Bi, value: Option<f64>, unit: &str, verdict: crate::reading::Bi) -> Reading {
    let meta = &m["meta"];
    Reading {
        id: id.into(), label, value, unit: unit.into(), verdict, key: false,
        source: "NASA POWER".into(),
        dataset: meta["dataset"].as_str().unwrap_or("POWER daily, AG community").into(),
        date: meta["period"].as_str().unwrap_or("").replace("..", " – "),
        resolution: meta["resolution"].as_str().unwrap_or("~0.5° × 0.625°").into(),
        badge: "regional context".into(),
        license: meta["license"].as_str().unwrap_or("NASA open data").into(),
        method: meta["method"].as_str().map(String::from),
        series: vec![],
    }
}

/// Readings from the POWER summary. `monthly` feeds the sparkline of the temperature reading.
pub fn readings(s: &Value, monthly: Option<&Value>, provisional: bool) -> Vec<Reading> {
    let mut out = vec![];
    if let Some(m) = metric(s, "last_30_days", "tmean_c") {
        if let (Some(v), Some(a)) = (m["value"].as_f64(), m["anomaly"].as_f64()) {
            let mut r = base(m, "temp_anomaly", bi("Temperature, last 30 days", "Temperatura, últimos 30 días"), Some(round(a, 1)), "°C vs normal", temp_verdict(a));
            r.key = true;
            let nw = &s["normals_window"];
            r.method = Some(format!("{:.1} °C mean vs 1985–2024 normal {:.1} °C over the same dates ({} to {}). {}",
                v, m["normal"]["mean"].as_f64().unwrap_or(f64::NAN), nw["from"].as_i64().unwrap_or(1985), nw["to"].as_i64().unwrap_or(2024),
                if provisional { "Latest weeks come from POWER near-real-time streams and are provisional." } else { "" }));
            if let Some(ms) = monthly.and_then(|x| x["months"].as_array()) {
                r.series = ms.iter().filter_map(|mo| {
                    let t = mo["metrics"].as_array()?.iter().find(|k| k["key"] == "tmean_c")?;
                    Some(json!({ "t": mo["month"], "v": t["value"] }))
                }).collect();
            }
            out.push(r);
        }
    }
    if let Some(m) = metric(s, "last_30_days", "rain_mm") {
        let v = m["value"].as_f64();
        let n = m["normal"]["mean"].as_f64().unwrap_or(0.0);
        let verdict = match v {
            Some(x) if x < 1.0 && n < 2.0 => bi("dry, as usual for this season", "seco, como es usual en esta época"),
            Some(x) if x > n * 1.5 + 2.0 => bi(format!("wetter than normal (normal {n:.0} mm)"), format!("más lluvioso que lo normal (normal {n:.0} mm)")),
            Some(x) if x < n * 0.5 => bi(format!("drier than normal (normal {n:.0} mm)"), format!("más seco que lo normal (normal {n:.0} mm)")),
            Some(_) => bi("close to normal", "cerca de lo normal"),
            None => bi("no data", "sin datos"),
        };
        out.push(base(m, "rain_30d", bi("Rain, last 30 days", "Lluvia, últimos 30 días"), v.map(|x| round(x, 1)), "mm", verdict));
    }
    if let Some(m) = metric(s, "last_30_days", "et0_mm") {
        let v = m["value"].as_f64();
        let a = m["anomaly"].as_f64().unwrap_or(0.0);
        out.push(base(m, "et0_30d", bi("Crop water demand (ET₀), last 30 days", "Demanda de agua (ET₀), últimos 30 días"), v.map(|x| round(x, 0)), "mm",
            bi(format!("{} mm vs normal: the air pulls {} water from the crop", signed(a, 0), if a > 5.0 { "more" } else if a < -5.0 { "less" } else { "about the usual" }),
               format!("{} mm vs lo normal: el aire extrae {} agua del cultivo", signed(a, 0), if a > 5.0 { "más" } else if a < -5.0 { "menos" } else { "lo usual de" }))));
    }
    if let Some(m) = metric(s, "last_30_days", "water_balance_mm") {
        let v = m["value"].as_f64();
        out.push(base(m, "water_balance", bi("Water balance (rain − ET₀), last 30 days", "Balance hídrico (lluvia − ET₀), últimos 30 días"), v.map(|x| round(x, 0)), "mm",
            bi("irrigation must cover this deficit (irrigation not modelled)", "el riego debe cubrir este déficit (riego no modelado)")));
    }
    if let Some(m) = metric(s, "season_to_date", "heat_days") {
        let v = m["value"].as_f64();
        let thr = s["options"]["heat_c"].as_f64().unwrap_or(32.0);
        out.push(base(m, "heat_days", bi(format!("Heat-stress days (≥ {thr:.0} °C) this season"), format!("Días de estrés por calor (≥ {thr:.0} °C) esta campaña")), v, "days",
            if v.unwrap_or(0.0) > 0.0 { bi("some heat-stress days", "algunos días de estrés por calor") } else { bi("none so far", "ninguno hasta ahora") }));
    }
    if let Some(m) = metric(s, "season_to_date", "gdd") {
        let a = m["anomaly"].as_f64().unwrap_or(0.0);
        out.push(base(m, "gdd", bi("Growing degree-days this season (base 10 °C)", "Grados-día de crecimiento esta campaña (base 10 °C)"), m["value"].as_f64().map(|x| round(x, 0)), "°C·day",
            bi(format!("{} vs normal: crops and pests develop {}", signed(a, 0), if a > 50.0 { "faster" } else if a < -50.0 { "slower" } else { "at the usual pace" }),
               format!("{} vs lo normal: cultivos y plagas se desarrollan {}", signed(a, 0), if a > 50.0 { "más rápido" } else if a < -50.0 { "más lento" } else { "al ritmo usual" }))));
    }
    if let Some(p) = s["pests"].as_array().and_then(|ps| ps.iter().find(|p| p["pest"]["key"].as_str().map(|k| k == "myzus_persicae").unwrap_or(false))) {
        let g = p["generations"].as_f64();
        let d = p["difference"].as_f64().unwrap_or(0.0);
        let meta = &p["meta"];
        out.push(Reading {
            id: "pest_gen".into(),
            label: bi("Green peach aphid generations this season", "Generaciones de pulgón verde esta campaña"),
            value: g.map(|x| round(x, 1)), unit: "generations".into(),
            verdict: bi(format!("{} vs normal (degree-days)", signed(d, 1)), format!("{} vs lo normal (grados-día)", signed(d, 1))),
            key: false, source: "NASA POWER + published threshold".into(),
            dataset: meta["dataset"].as_str().unwrap_or("").into(),
            date: meta["period"].as_str().unwrap_or("").replace("..", " – "),
            resolution: "~0.5° × 0.625°".into(), badge: "regional context".into(),
            license: meta["license"].as_str().unwrap_or("").into(),
            method: Some(format!("Single-sine degree-days, {} °C lower threshold, {} °C·day per generation ({} {}).",
                p["pest"]["lower_c"], p["pest"]["dd_per_generation"], p["pest"]["citation"]["authors"].as_str().unwrap_or(""), p["pest"]["citation"]["year"])),
            series: vec![],
        });
    }
    out
}

/// Pest rows for the evidence package.
pub fn pests(s: &Value) -> Vec<Value> {
    s["pests"].as_array().into_iter().flatten().map(|p| {
        let c = &p["pest"]["citation"];
        json!({
            "name": p["pest"]["common_name"], "scientific": p["pest"]["scientific_name"],
            "generations_now": p["generations"].as_f64().map(|x| round(x, 1)),
            "generations_normal": p["normal_generations"]["mean"].as_f64().map(|x| round(x, 1)),
            "citation": format!("{} ({}). {}. {}. {}", c["authors"].as_str().unwrap_or(""), c["year"], c["title"].as_str().unwrap_or(""), c["published_in"].as_str().unwrap_or(""), c["url"].as_str().unwrap_or("")),
        })
    }).collect()
}

/// Monthly rows: this year vs the 1985–2024 band for the same dates.
pub fn climate_monthly(m: &Value) -> Vec<Value> {
    m["months"].as_array().into_iter().flatten().filter_map(|mo| {
        let ms = mo["metrics"].as_array()?;
        let t = ms.iter().find(|k| k["key"] == "tmean_c")?;
        let r = ms.iter().find(|k| k["key"] == "rain_mm");
        Some(json!({
            "month": mo["month"], "partial": mo["partial"],
            "t_mean": t["value"], "normal_mean": t["normal"]["mean"], "normal_lo": t["normal"]["p10"], "normal_hi": t["normal"]["p90"],
            "rain": r.map(|r| r["value"].clone()), "normal_rain": r.map(|r| r["normal"]["mean"].clone()),
        }))
    }).collect()
}

/// True when the newest POWER days come from near-real-time streams (GEOS-IT / FLASHFlux), i.e. provisional.
pub fn provisional(s: &Value) -> bool {
    s.to_string().contains("GEOS-IT") || s.to_string().contains("FLASHFlux") || s.to_string().contains("FLASHFLUX")
}
