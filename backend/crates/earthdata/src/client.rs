//! HTTP plumbing: CMR granule search (no auth), OPeNDAP `.dap.csv` reads with the Earthdata bearer token,
//! the DAP CSV parser and a small bounded-concurrency helper.
//! The token is only ever placed in the Authorization header; errors are built without URLs or headers.
use anyhow::{anyhow, bail, Context};
use serde::Deserialize;
use std::{collections::HashMap, future::Future, sync::Arc};

pub const CMR: &str = "https://cmr.earthdata.nasa.gov/search/granules.json";

#[derive(Clone, Debug, Deserialize)]
pub struct CmrGranule {
    /// GranuleUR, e.g. `SMAP_L4_SM_gph_20261110T223000_Vv8013_001.h5_u9loH5HD`.
    pub title: String,
    #[serde(default)]
    pub producer_granule_id: Option<String>,
    pub time_start: String,
    #[serde(default)]
    pub collection_concept_id: Option<String>,
}

#[derive(Deserialize)]
struct CmrFeed { feed: CmrEntries }
#[derive(Deserialize)]
struct CmrEntries { entry: Vec<CmrGranule> }

/// Granules of a collection whose time overlaps any of the `temporal` ranges ("start,end" in ISO 8601), oldest first.
pub async fn cmr_granules(http: &reqwest::Client, short_name: &str, version: &str, temporal: &[String]) -> anyhow::Result<Vec<CmrGranule>> {
    let mut q: Vec<(&str, String)> = vec![
        ("short_name", short_name.into()),
        ("version", version.into()),
        ("sort_key", "start_date".into()),
        ("page_size", "2000".into()),
    ];
    let key = if temporal.len() > 1 { "temporal[]" } else { "temporal" };
    for t in temporal {
        q.push((key, t.clone()));
    }
    let resp = http.get(CMR).query(&q).send().await.map_err(|e| anyhow!("CMR search failed: {}", e.without_url()))?;
    if !resp.status().is_success() {
        bail!("CMR search returned HTTP {}", resp.status().as_u16());
    }
    let feed: CmrFeed = resp.json().await.map_err(|e| anyhow!("CMR response unreadable: {}", e.without_url()))?;
    Ok(feed.feed.entry)
}

/// Percent-encodes a DAP4 constraint expression (brackets, colons, slashes, semicolons).
pub fn encode_ce(ce: &str) -> String {
    let mut out = String::with_capacity(ce.len() * 2);
    for b in ce.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// GET `<granule_url>.dap.csv?dap4.ce=<ce>` with the bearer token and returns the parsed variables.
pub async fn dap_csv(http: &reqwest::Client, token: &str, granule_url: &str, ce: &str, what: &str) -> anyhow::Result<DapCsv> {
    let url = format!("{granule_url}.dap.csv?dap4.ce={}", encode_ce(ce));
    let resp = http
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow!("{what} OPeNDAP request failed: {}", e.without_url()))?;
    let status = resp.status();
    if !status.is_success() {
        bail!("{what} OPeNDAP returned HTTP {}{}", status.as_u16(),
            if status.as_u16() == 401 { " (Earthdata token missing, expired or the data archive not authorized)" } else { "" });
    }
    let text = resp.text().await.map_err(|e| anyhow!("{what} OPeNDAP body unreadable: {}", e.without_url()))?;
    if text.starts_with("Dataset: ERROR") || text.trim_start().starts_with('<') {
        bail!("{what} OPeNDAP returned an error document");
    }
    Ok(DapCsv::parse(&text))
}

/// Parsed DAP CSV: lines like `/Geophysical_Data/sm_surface[0], 0.051, 0.054` keyed by the text before the first comma.
#[derive(Debug, Default)]
pub struct DapCsv(pub HashMap<String, Vec<f64>>);

impl DapCsv {
    pub fn parse(text: &str) -> Self {
        let mut map = HashMap::new();
        for line in text.lines() {
            let Some((name, rest)) = line.split_once(',') else { continue };
            if !name.starts_with('/') { continue; }
            let vals: Vec<f64> = rest.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            if !vals.is_empty() {
                map.insert(name.trim().to_string(), vals);
            }
        }
        Self(map)
    }

    /// Value at row `r` (the bracketed index after the variable name) and column `c` of the returned block.
    pub fn at(&self, var: &str, r: usize, c: usize) -> Option<f64> {
        self.0.get(&format!("{var}[{r}]")).and_then(|v| v.get(c)).copied()
    }

    /// For 3-D blocks like `/precipitation[0][0]`.
    pub fn at3(&self, var: &str, t: usize, r: usize, c: usize) -> Option<f64> {
        self.0.get(&format!("{var}[{t}][{r}]")).and_then(|v| v.get(c)).copied()
    }
}

/// Treats fill values (≤ -999) and non-finite numbers as missing.
pub fn valid(v: Option<f64>) -> Option<f64> {
    v.filter(|x| x.is_finite() && *x > -999.0)
}

/// Runs `f` over `items` with at most `limit` tasks in flight; results keep the input order.
pub async fn bounded<T, R, F, Fut>(items: Vec<T>, limit: usize, f: F) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
{
    let sem = Arc::new(tokio::sync::Semaphore::new(limit.max(1)));
    let f = Arc::new(f);
    let mut set = tokio::task::JoinSet::new();
    let n = items.len();
    for (i, item) in items.into_iter().enumerate() {
        let (sem, f) = (sem.clone(), f.clone());
        set.spawn(async move {
            let _permit = sem.acquire_owned().await;
            (i, f(item).await)
        });
    }
    let mut out: Vec<Option<R>> = (0..n).map(|_| None).collect();
    while let Some(res) = set.join_next().await {
        if let Ok((i, r)) = res {
            out[i] = Some(r);
        }
    }
    out.into_iter().flatten().collect()
}

/// Reads the Earthdata token from the app config or fails with a clear message (never echoing it).
pub fn token(state: &common::AppState) -> anyhow::Result<String> {
    state.config.secrets.earthdata.clone().filter(|t| !t.trim().is_empty())
        .context("Earthdata token missing: add the `earthdata <token>` line to env.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dap_csv_blocks() {
        let text = "Dataset: x.nc4\n/lon, -77.25, -77.15\n/precipitation[0][0], 0.195, 0\n/precipitation[0][1], 0.035, 0.24\n";
        let d = DapCsv::parse(text);
        assert_eq!(d.at3("/precipitation", 0, 0, 0), Some(0.195));
        assert_eq!(d.at3("/precipitation", 0, 1, 1), Some(0.24));
        let s = DapCsv::parse("/Geophysical_Data/sm_surface[0], -9999, 0.05\n");
        assert_eq!(valid(s.at("/Geophysical_Data/sm_surface", 0, 0)), None);
        assert_eq!(valid(s.at("/Geophysical_Data/sm_surface", 0, 1)), Some(0.05));
    }

    #[test]
    fn encodes_constraint() {
        assert_eq!(encode_ce("/a/b[1:2];/c"), "%2Fa%2Fb%5B1%3A2%5D%3B%2Fc");
    }
}
