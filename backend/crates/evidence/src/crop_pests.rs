//! Generations of the current crop's own pests, from their published degree-day thresholds (crops knowledge base)
//! and NASA POWER daily temperature: this season vs the same dates in 1985-2024. Single-sine method, no upper cutoff
//! unless published (Zalom et al. 1983). Regional context (~50 km POWER cell).
use common::AppState;
use power::{days, gdd};
use serde_json::{json, Value};

pub async fn generations(state: &AppState, lat: f64, lon: f64, thresholds: &[Value]) -> anyhow::Result<Vec<Value>> {
    if thresholds.is_empty() { return Ok(vec![]); }
    let d = power::daily(state, lat, lon).await?;
    let all = days::days(&d);
    let end = days::last_complete(&all).ok_or_else(|| anyhow::anyhow!("no complete POWER days"))?;
    let start = days::default_season_start(lat, end);
    let sum = |from, to, tl: f64, dd: f64| -> Option<f64> {
        let s = days::slice(&all, from, to);
        let v: Vec<f64> = s.iter().filter_map(|x| Some(gdd::dd_sine(x.tmin?, x.tmax?, tl, None))).collect();
        (v.len() as f64 >= 0.9 * ((to - from).num_days() + 1) as f64).then(|| v.iter().sum::<f64>() / dd)
    };
    let mut out = vec![];
    for t in thresholds {
        let (Some(tl), Some(dd)) = (t["lower_threshold_c"].as_f64(), t["degree_days"].as_f64()) else { continue };
        if dd <= 0.0 { continue; }
        let now = sum(start, end, tl, dd);
        let normals: Vec<f64> = (1985..=2024).filter_map(|y| { let (f, e) = days::same_window(start, end, y); sum(f, e, tl, dd) }).collect();
        let normal = (!normals.is_empty()).then(|| normals.iter().sum::<f64>() / normals.len() as f64);
        let r1 = |x: f64| (x * 10.0).round() / 10.0;
        out.push(json!({
            "name": t["name"], "scientific": t["scientific_name"],
            "lower_threshold_c": tl, "degree_days": dd,
            "generations_now": now.map(r1), "generations_normal": normal.map(r1),
            "difference": now.zip(normal).map(|(a, b)| r1(a - b)),
            "season": format!("{start} – {end}"),
            "citation": t["degree_days_citation"], "threshold_citation": t["threshold_citation"], "note": t["note"],
        }));
    }
    Ok(out)
}
