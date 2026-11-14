//! GEOS-FP OPeNDAP access: run discovery, grid indexing, 48-hour block reads and daily aggregation.
use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

const BASE: &str = "https://opendap.nccs.nasa.gov/dods/GEOS-5/fp/0.25_deg/fcast";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Day {
    pub date: String,
    pub tmin: f64,
    pub tmax: f64,
    pub tmean: f64,
    pub rain_mm: f64,
}

/// Grid index on the 0.25° (lat) × 0.3125° (lon) GEOS-FP grid that starts at -90, -180.
pub fn index(lat: f64, lon: f64) -> (usize, usize) {
    (((lat + 90.0) / 0.25).round() as usize, ((lon + 180.0) / 0.3125).round().rem_euclid(1152.0) as usize)
}

pub fn center(i: usize, j: usize) -> [f64; 2] { [i as f64 * 0.25 - 90.0, j as f64 * 0.3125 - 180.0] }

/// "tavg1_2d_slv_Nx.20261114_00" → "2026-11-14 00 UTC".
pub fn run_label(run: &str) -> String {
    let id = run.rsplit('.').next().unwrap_or(run);
    if id.len() >= 11 { format!("{}-{}-{} {} UTC", &id[0..4], &id[4..6], &id[6..8], &id[9..11]) } else { id.to_string() }
}

/// The newest 00 UTC run (the 00 UTC runs carry the full 10-day horizon; other runs are short).
pub async fn latest_full_run(http: &reqwest::Client) -> anyhow::Result<String> {
    let html = http.get(format!("{BASE}/tavg1_2d_slv_Nx/")).send().await?.error_for_status()?.text().await?;
    // Entries look like "tavg1_2d_slv_Nx.20261114_00" followed by ':' or '.info' etc.: take the 11-character run id.
    let tag = "tavg1_2d_slv_Nx.";
    let mut runs: Vec<String> = html.match_indices(tag)
        .filter_map(|(i, _)| html.get(i + tag.len()..i + tag.len() + 11))
        .filter(|id| id.len() == 11 && id.as_bytes()[8] == b'_' && id.ends_with("_00") && id[..8].bytes().all(|c| c.is_ascii_digit()))
        .map(|id| format!("{tag}{id}")).collect();
    runs.sort();
    runs.dedup();
    runs.pop().ok_or_else(|| anyhow::anyhow!("no GEOS-FP 00 UTC run listed"))
}

fn parse_ascii(text: &str) -> Vec<f64> {
    text.lines().filter_map(|l| {
        let l = l.trim();
        if !l.starts_with('[') { return None; }
        l.rsplit(',').next()?.trim().parse::<f64>().ok()
    }).collect()
}

async fn block(http: reqwest::Client, url: String, what: String) -> anyhow::Result<Vec<f64>> {
    for attempt in 0..3 {
        // Back off before a retry: the server refuses connections when it is busy.
        if attempt > 0 { tokio::time::sleep(std::time::Duration::from_millis(1500 * attempt)).await; }
        match http.get(&url).timeout(std::time::Duration::from_secs(60)).send().await.and_then(|r| r.error_for_status()) {
            Ok(r) => {
                let vals = parse_ascii(&r.text().await?);
                if vals.len() >= 48 { return Ok(vals.into_iter().take(48).collect()); }
                tracing::warn!("geos-fp {what}: {} values (attempt {attempt})", vals.len());
            }
            Err(e) => tracing::warn!("geos-fp {what} attempt {attempt}: {}", e.without_url()),
        }
    }
    anyhow::bail!("GEOS-FP server did not answer for {what}")
}

/// Five 48-hour blocks read in parallel (the server times out on longer ranges).
async fn read(http: &reqwest::Client, collection: &str, run: &str, var: &str, i: usize, j: usize) -> anyhow::Result<Vec<f64>> {
    let run_id = run.rsplit('.').next().unwrap_or(run);
    let mut set = tokio::task::JoinSet::new();
    for (k, start) in (0..240).step_by(48).enumerate() {
        let url = format!("{BASE}/{collection}/{collection}.{run_id}.ascii?{var}%5B{start}:{}%5D%5B{i}%5D%5B{j}%5D", start + 47);
        let fut = block(http.clone(), url, format!("{var} block {start}"));
        set.spawn(async move { (k, fut.await) });
    }
    let mut parts: Vec<(usize, Vec<f64>)> = vec![];
    while let Some(r) = set.join_next().await {
        let (k, v) = r?;
        parts.push((k, v?));
    }
    parts.sort_by_key(|p| p.0);
    Ok(parts.into_iter().flat_map(|p| p.1).collect())
}

/// Hourly t2m (K) and prectot (kg m-2 s-1) of the run → local solar days (UTC + lon/15 h), complete days only.
pub async fn daily(http: &reqwest::Client, run: &str, i: usize, j: usize, lon: f64) -> anyhow::Result<Vec<Day>> {
    let (t2m, rain) = tokio::try_join!(read(http, "tavg1_2d_slv_Nx", run, "t2m", i, j), read(http, "tavg1_2d_flx_Nx", run, "prectot", i, j))?;
    let id = run.rsplit('.').next().unwrap_or(run);
    let t0 = NaiveDateTime::parse_from_str(&format!("{}0000", &id[0..11].replace('_', "")), "%Y%m%d%H%M%S")?;
    let offset = Duration::minutes((lon / 15.0 * 60.0).round() as i64);
    let mut days: Vec<(String, Vec<f64>, f64)> = vec![];
    for (h, (t, p)) in t2m.iter().zip(rain.iter()).enumerate() {
        let local = t0 + Duration::minutes(30 + 60 * h as i64) + offset;
        let date = local.date().to_string();
        match days.last_mut() {
            Some((d, ts, r)) if *d == date => { ts.push(*t - 273.15); *r += p * 3600.0; }
            _ => days.push((date, vec![*t - 273.15], p * 3600.0)),
        }
    }
    let r1 = |x: f64| (x * 10.0).round() / 10.0;
    Ok(days.into_iter().filter(|(_, ts, _)| ts.len() >= 20).map(|(date, ts, r)| Day {
        date,
        tmin: r1(ts.iter().cloned().fold(f64::MAX, f64::min)),
        tmax: r1(ts.iter().cloned().fold(f64::MIN, f64::max)),
        tmean: r1(ts.iter().sum::<f64>() / ts.len() as f64),
        rain_mm: r1(r.max(0.0)),
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn huaral_index_matches_the_server() {
        assert_eq!(index(-11.48, -77.18), (314, 329));
        assert_eq!(run_label("tavg1_2d_slv_Nx.20261114_00"), "2026-11-14 00 UTC");
        assert_eq!(parse_ascii("t2m, [2][1][1]\n[0][0], 295.0\n\n[1][0], 296.5\n"), vec![295.0, 296.5]);
    }
}
