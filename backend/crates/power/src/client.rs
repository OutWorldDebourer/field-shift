//! NASA POWER daily point client: one request from 1981-01-01 to today (about 2.5 MB, ~3 s, so no decade split is
//! needed), fill value -999 mapped to missing, stored column-wise so the cached JSON stays small.
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const BASE_URL: &str = "https://power.larc.nasa.gov/api/temporal/daily/point";
pub const PARAMETERS: &str = "T2M,T2M_MAX,T2M_MIN,PRECTOTCORR,RH2M,WS2M,ALLSKY_SFC_SW_DWN,T2MDEW,PS";
pub const FIRST_DAY: (i32, u32, u32) = (1981, 1, 1);

/// Daily POWER values for one point, one vector per parameter, index 0 = `start`. Missing values are `None`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerDaily {
    pub lat: f64,
    pub lon: f64,
    /// POWER cell elevation in metres (third coordinate of the response geometry).
    pub elevation_m: f64,
    pub start: NaiveDate,
    pub sources: Vec<String>,
    pub api_version: String,
    pub fetched_at: DateTime<Utc>,
    pub t2m: Vec<Option<f64>>,
    pub t2m_max: Vec<Option<f64>>,
    pub t2m_min: Vec<Option<f64>>,
    pub prectotcorr: Vec<Option<f64>>,
    pub rh2m: Vec<Option<f64>>,
    pub ws2m: Vec<Option<f64>>,
    pub allsky_sfc_sw_dwn: Vec<Option<f64>>,
    pub t2mdew: Vec<Option<f64>>,
    pub ps: Vec<Option<f64>>,
}

impl PowerDaily {
    pub fn len(&self) -> usize { self.t2m.len() }
    pub fn is_empty(&self) -> bool { self.t2m.is_empty() }
    pub fn date(&self, i: usize) -> NaiveDate { self.start + Duration::days(i as i64) }
}

/// Coordinates rounded to 0.01° (about 1 km, far finer than the POWER cell) for the request and the cache key.
pub fn round_coord(x: f64) -> f64 { (x * 100.0).round() / 100.0 }

pub fn request_url(lat: f64, lon: f64, start: NaiveDate, end: NaiveDate) -> String {
    format!(
        "{BASE_URL}?parameters={PARAMETERS}&community=AG&longitude={lon}&latitude={lat}&start={}&end={}&format=JSON",
        start.format("%Y%m%d"), end.format("%Y%m%d")
    )
}

/// Fetches the full daily series 1981-01-01..`end` straight from POWER (no cache). One retry on failure.
pub async fn fetch_daily(http: &reqwest::Client, lat: f64, lon: f64, end: NaiveDate) -> anyhow::Result<PowerDaily> {
    let (lat, lon) = (round_coord(lat), round_coord(lon));
    let start = NaiveDate::from_ymd_opt(FIRST_DAY.0, FIRST_DAY.1, FIRST_DAY.2).unwrap();
    let url = request_url(lat, lon, start, end);
    let mut last_err = None;
    for attempt in 0..2 {
        if attempt > 0 { tokio::time::sleep(std::time::Duration::from_secs(2)).await; }
        match get_json(http, &url).await {
            Ok(v) => return parse(&v, lat, lon),
            Err(e) => { tracing::warn!("POWER daily attempt {} failed: {e}", attempt + 1); last_err = Some(e); }
        }
    }
    Err(last_err.unwrap())
}

async fn get_json(http: &reqwest::Client, url: &str) -> anyhow::Result<serde_json::Value> {
    let resp = http.get(url).send().await.map_err(|e| anyhow::anyhow!("POWER request: {}", e.without_url()))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("POWER returned {status}: {}", body.chars().take(300).collect::<String>());
    }
    resp.json().await.map_err(|e| anyhow::anyhow!("POWER JSON: {}", e.without_url()))
}

/// Parses a POWER daily point GeoJSON response; trailing days where every parameter is missing are dropped.
pub fn parse(v: &serde_json::Value, lat: f64, lon: f64) -> anyhow::Result<PowerDaily> {
    let fill = v["header"]["fill_value"].as_f64().unwrap_or(-999.0);
    let params = v["properties"]["parameter"].as_object()
        .ok_or_else(|| anyhow::anyhow!("POWER response has no properties.parameter"))?;
    let col = |name: &str| -> BTreeMap<NaiveDate, Option<f64>> {
        params.get(name).and_then(|p| p.as_object()).map(|m| {
            m.iter().filter_map(|(k, val)| {
                let d = NaiveDate::parse_from_str(k, "%Y%m%d").ok()?;
                let x = val.as_f64().filter(|x| (*x - fill).abs() > 1e-6 && *x > -998.0);
                Some((d, x))
            }).collect()
        }).unwrap_or_default()
    };
    let names = ["T2M", "T2M_MAX", "T2M_MIN", "PRECTOTCORR", "RH2M", "WS2M", "ALLSKY_SFC_SW_DWN", "T2MDEW", "PS"];
    let cols: Vec<BTreeMap<NaiveDate, Option<f64>>> = names.iter().map(|n| col(n)).collect();
    let start = cols[0].keys().next().copied().ok_or_else(|| anyhow::anyhow!("POWER response has no T2M days"))?;
    let mut end = *cols[0].keys().last().unwrap();
    while end > start && cols.iter().all(|c| c.get(&end).copied().flatten().is_none()) {
        end = end.pred_opt().unwrap();
    }
    let n = (end - start).num_days() as usize + 1;
    let vec_of = |c: &BTreeMap<NaiveDate, Option<f64>>| -> Vec<Option<f64>> {
        (0..n).map(|i| c.get(&(start + Duration::days(i as i64))).copied().flatten()).collect()
    };
    let mut it = cols.iter().map(vec_of);
    Ok(PowerDaily {
        lat,
        lon,
        elevation_m: v["geometry"]["coordinates"][2].as_f64().unwrap_or(0.0),
        start,
        sources: v["header"]["sources"].as_array().map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect()).unwrap_or_default(),
        api_version: v["header"]["api"]["version"].as_str().unwrap_or("").to_string(),
        fetched_at: Utc::now(),
        t2m: it.next().unwrap(),
        t2m_max: it.next().unwrap(),
        t2m_min: it.next().unwrap(),
        prectotcorr: it.next().unwrap(),
        rh2m: it.next().unwrap(),
        ws2m: it.next().unwrap(),
        allsky_sfc_sw_dwn: it.next().unwrap(),
        t2mdew: it.next().unwrap(),
        ps: it.next().unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fill_values_and_trims_tail() {
        let v = serde_json::json!({
            "geometry": {"coordinates": [-77.21, -11.47, 224.43]},
            "header": {"fill_value": -999.0, "sources": ["MERRA2"], "api": {"version": "v2.10.3"}},
            "properties": {"parameter": {
                "T2M": {"20250101": 20.0, "20250102": -999.0, "20250103": -999.0},
                "T2M_MAX": {"20250101": 24.0, "20250102": 25.0, "20250103": -999.0},
                "ALLSKY_SFC_SW_DWN": {"20250101": -999.0, "20250102": -999.0, "20250103": -999.0}
            }}
        });
        let d = parse(&v, -11.47, -77.21).unwrap();
        assert_eq!(d.len(), 2);
        assert_eq!(d.elevation_m, 224.43);
        assert_eq!(d.t2m, vec![Some(20.0), None]);
        assert_eq!(d.t2m_max, vec![Some(24.0), Some(25.0)]);
        assert_eq!(d.allsky_sfc_sw_dwn, vec![None, None]);
        assert_eq!(d.ps, vec![None, None]);
    }
}
