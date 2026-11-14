//! SMAP soil moisture, IMERG rain and GRACE water storage → regional-context readings (Earthdata Login sources).
use crate::reading::{bi, round, signed, Reading};
use serde_json::{json, Value};

fn base(p: &Value, id: &str, label: crate::reading::Bi, value: Option<f64>, unit: &str, verdict: crate::reading::Bi, date: String) -> Reading {
    Reading {
        id: id.into(), label, value, unit: unit.into(), verdict, key: false,
        source: match id { "soil_moisture" => "NASA SMAP L4", "rain_imerg" => "NASA GPM IMERG Late", "grace" => "NASA GRACE/GRACE-FO", _ => "NASA Earthdata" }.into(),
        dataset: format!("{} v{}", p["dataset_id"].as_str().unwrap_or(""), p["version"].as_str().unwrap_or("")),
        date, resolution: p["resolution"].as_str().unwrap_or("").into(),
        badge: "regional context".into(),
        license: format!("{} DOI {}", p["license"].as_str().unwrap_or(""), p["doi"].as_str().unwrap_or("")),
        method: None, series: vec![], provisional: false, note: None,
    }
}

pub fn smap(s: &Value) -> Option<Reading> {
    let v = s["latest"]["surface_m3m3"].as_f64()?;
    let anom = s["comparison"]["surface_anomaly_m3m3"].as_f64();
    let years = s["comparison"]["years"].as_array().map(|a| a.len()).or(s["comparison"]["years"].as_u64().map(|x| x as usize)).unwrap_or(0);
    let verdict = match anom {
        Some(a) if a <= -0.02 => bi(format!("drier than the same date in {years} past years"), format!("más seco que la misma fecha en {years} años anteriores")),
        Some(a) if a >= 0.02 => bi(format!("wetter than the same date in {years} past years"), format!("más húmedo que la misma fecha en {years} años anteriores")),
        Some(_) => bi("close to the same date in past years", "similar a la misma fecha en años anteriores"),
        None => bi("no past-year comparison", "sin comparación con años anteriores"),
    };
    let mut r = base(&s["provenance"], "soil_moisture", bi("Soil moisture of the surrounding area (top 5 cm)", "Humedad del suelo de la zona (5 cm superiores)"),
        Some(round(v, 3)), "m³/m³", verdict, s["latest"]["time"].as_str().unwrap_or("").chars().take(10).collect());
    r.method = Some(format!("SMAP L4 9 km model-assimilation cell; anomaly {} m³/m³ vs the same date in earlier years. Describes the desert and hills around the valley, not an irrigated plot.",
        anom.map(|a| signed(a, 3)).unwrap_or_else(|| "n/a".into())));
    r.series = s["daily"].as_array().into_iter().flatten().map(|d| json!({ "t": d["time"].as_str().unwrap_or("").get(..10), "v": d["surface_m3m3"] })).collect();
    Some(r)
}

pub fn imerg(s: &Value) -> Option<Reading> {
    let sum = &s["summary"];
    let total = sum["total_mm"].as_f64()?;
    let rainy = sum["rainy_days"].as_u64().unwrap_or(0);
    let verdict = if total < 1.0 { bi("almost no rain, as usual on the desert coast", "casi sin lluvia, como es usual en la costa desértica") }
        else { bi(format!("{rainy} rainy day(s)"), format!("{rainy} día(s) con lluvia")) };
    let mut r = base(&s["provenance"], "rain_imerg", bi("Satellite rain estimate, last 30 days", "Lluvia estimada por satélite, últimos 30 días"),
        Some(round(total, 1)), "mm", verdict, format!("{} – {}", sum["first_date"].as_str().unwrap_or(""), sum["last_date"].as_str().unwrap_or("")));
    r.method = Some("Sum of GPM IMERG Late daily estimates for the ~10 km cell; light coastal drizzle is often missed.".into());
    r.series = s["daily"].as_array().into_iter().flatten().map(|d| json!({ "t": d["date"], "v": d["rain_mm"] })).collect();
    Some(r)
}

pub fn grace(s: &Value) -> Option<Reading> {
    if s["available"] != true { return None; }
    let st = &s["stats"];
    let trend = st["trend_cm_per_year"].as_f64()?;
    let verdict = if trend > 0.1 { bi(format!("slowly wetting since 2002 ({} cm/yr)", signed(trend, 2)), format!("se humedece lentamente desde 2002 ({} cm/año)", signed(trend, 2))) }
        else if trend < -0.1 { bi(format!("slowly drying since 2002 ({} cm/yr)", signed(trend, 2)), format!("se seca lentamente desde 2002 ({} cm/año)", signed(trend, 2))) }
        else { bi("no clear long-term trend", "sin tendencia clara a largo plazo") };
    let mut r = base(&s["provenance"], "grace", bi("Water stored in the region (GRACE)", "Agua almacenada en la región (GRACE)"),
        st["latest_cm"].as_f64().map(|x| round(x, 1)), "cm", verdict,
        format!("{} – {}", st["first_date"].as_str().unwrap_or(""), st["last_date"].as_str().unwrap_or("")));
    r.method = Some("Total water storage anomaly (soil, groundwater, snow) for a ~300 km mascon; regional context for long-term change only.".into());
    r.series = s["series"].as_array().map(|a| a.iter().rev().take(36).rev().map(|d| json!({ "t": d["date"], "v": d["lwe_cm"] })).collect()).unwrap_or_default();
    Some(r)
}
