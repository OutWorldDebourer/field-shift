//! SMAP L4 (SPL4SMGP v008) surface and root-zone soil moisture at a point: latest 3-hourly value, one value per day
//! for the recent window, and the same date and hour in every earlier SMAP year (2015 on) for comparison.
use crate::{client::{self, valid, CmrGranule}, ctx::Ctx, grid::{self, GridCell}, meta::Provenance};
use anyhow::{bail, Context};
use chrono::{Datelike, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SHORT_NAME: &str = "SPL4SMGP";
const VERSION: &str = "008";
const COLLECTION: &str = "C3480440870-NSIDC_CPRD";
const OPENDAP: &str = "https://opendap.earthdata.nasa.gov/collections";
const FIRST_YEAR: i32 = 2015;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmapReading {
    /// Granule nominal time (UTC), ISO 8601.
    pub time: String,
    pub granule: String,
    /// Volumetric water content of the top 0-5 cm, m³/m³.
    pub surface_m3m3: Option<f64>,
    /// Volumetric water content of the root zone 0-100 cm, m³/m³.
    pub rootzone_m3m3: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SameDateComparison {
    pub years: Vec<i32>,
    pub surface_mean_m3m3: Option<f64>,
    pub rootzone_mean_m3m3: Option<f64>,
    /// latest - mean of earlier years (negative = drier than usual for this date).
    pub surface_anomaly_m3m3: Option<f64>,
    pub rootzone_anomaly_m3m3: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmapReport {
    pub provenance: Provenance,
    pub requested: common::geo::LatLon,
    pub cell: GridCell,
    pub units: String,
    pub latest: Option<SmapReading>,
    /// One reading per UTC day (the day's last 3-hourly granule), oldest first.
    pub daily: Vec<SmapReading>,
    /// Same calendar date and hour as `latest` in each earlier year.
    pub same_date_past_years: Vec<SmapReading>,
    pub comparison: Option<SameDateComparison>,
    pub note: String,
    pub gaps: Vec<String>,
}

/// Nominal time from a SMAP L4 granule name: `SMAP_L4_SM_gph_20261110T223000_...` → 2026-11-10T22:30:00.
pub fn granule_time(title: &str) -> Option<NaiveDateTime> {
    let i = title.find("_gph_")? + 5;
    NaiveDateTime::parse_from_str(title.get(i..i + 15)?, "%Y%m%dT%H%M%S").ok()
}

fn granule_url(g: &CmrGranule) -> String {
    let coll = g.collection_concept_id.as_deref().unwrap_or(COLLECTION);
    format!("{OPENDAP}/{coll}/granules/{}", g.title)
}

/// Reads surface and root-zone moisture of one cell from one granule (2×2 block, the cell is the first element).
pub async fn read_cell(ctx: &Ctx, g: &CmrGranule, cell: GridCell) -> anyhow::Result<SmapReading> {
    let r0 = cell.row.min(grid::EASE_ROWS - 2);
    let c0 = cell.col.min(grid::EASE_COLS - 2);
    let (dr, dc) = (cell.row - r0, cell.col - c0);
    let ce = format!(
        "/Geophysical_Data/sm_surface[{r0}:{r1}][{c0}:{c1}];/Geophysical_Data/sm_rootzone[{r0}:{r1}][{c0}:{c1}]",
        r1 = r0 + 1, c1 = c0 + 1
    );
    let key = format!("earthdata:smap:{}:{}:{}", g.title, cell.row, cell.col);
    let (http, token, url, title) = (ctx.http.clone(), ctx.token.clone(), granule_url(g), g.title.clone());
    let time = granule_time(&g.title).map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string()).unwrap_or_default();
    ctx.cached("smap", &key, 24 * 365, || async move {
        let d = client::dap_csv(&http, &token, &url, &ce, "SMAP").await?;
        let surface = valid(d.at("/Geophysical_Data/sm_surface", dr, dc));
        let rootzone = valid(d.at("/Geophysical_Data/sm_rootzone", dr, dc));
        if surface.is_none() && rootzone.is_none() && d.0.is_empty() {
            bail!("SMAP returned no values for this cell");
        }
        Ok(SmapReading { time, granule: title, surface_m3m3: surface.map(r4), rootzone_m3m3: rootzone.map(r4) })
    })
    .await
}

fn r4(x: f64) -> f64 { (x * 10_000.0).round() / 10_000.0 }

fn mean(xs: impl Iterator<Item = f64>) -> Option<f64> {
    let v: Vec<f64> = xs.collect();
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// Full SMAP report for a point. `days` is clamped to 1..=30.
pub async fn report(ctx: &Ctx, lat: f64, lon: f64, days: u32) -> anyhow::Result<SmapReport> {
    let days = days.clamp(1, 30) as i64;
    let cell = grid::ease9_cell(lat, lon).context("point outside the EASE-Grid 2.0 global grid (|lat| > ~85°)")?;
    let mut gaps = Vec::new();

    // Recent granules (SMAP L4 lags ~3-4 days); one per UTC day, the day's last one.
    let now = Utc::now().naive_utc();
    let start = now - Duration::days(days + 6);
    let temporal = vec![format!("{},{}", start.format("%Y-%m-%dT%H:%M:%SZ"), now.format("%Y-%m-%dT%H:%M:%SZ"))];
    let recent = client::cmr_granules(&ctx.http, SHORT_NAME, VERSION, &temporal).await?;
    let mut per_day: BTreeMap<chrono::NaiveDate, CmrGranule> = BTreeMap::new();
    for g in recent {
        if let Some(t) = granule_time(&g.title) {
            let keep = per_day.get(&t.date()).and_then(|o| granule_time(&o.title)).map_or(true, |ot| t > ot);
            if keep { per_day.insert(t.date(), g); }
        }
    }
    let chosen: Vec<CmrGranule> = per_day.into_values().rev().take(days as usize).collect::<Vec<_>>().into_iter().rev().collect();
    let Some(latest_g) = chosen.last().cloned() else { bail!("no SMAP L4 granules found in CMR for the last {days} days") };
    let lt = granule_time(&latest_g.title).context("unexpected SMAP granule name")?;

    // Same date and hour in earlier years: one CMR call with several temporal ranges.
    let past_times: Vec<NaiveDateTime> = (FIRST_YEAR..lt.year())
        .filter_map(|y| lt.date().with_year(y).map(|d| d.and_time(lt.time())))
        .filter(|t| t.date() >= chrono::NaiveDate::from_ymd_opt(2015, 3, 31).unwrap())
        .collect();
    let mut past_granules = Vec::new();
    if !past_times.is_empty() {
        let ranges: Vec<String> = past_times.iter().map(|t| {
            let s = t.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            format!("{s},{s}")
        }).collect();
        match client::cmr_granules(&ctx.http, SHORT_NAME, VERSION, &ranges).await {
            Ok(gs) => past_granules = gs.into_iter().filter(|g| granule_time(&g.title).is_some_and(|t| past_times.contains(&t))).collect(),
            Err(e) => gaps.push(format!("same-date comparison unavailable: {e}")),
        }
    }

    let n_recent = chosen.len();
    let all: Vec<CmrGranule> = chosen.into_iter().chain(past_granules).collect();
    let ctx2 = ctx.clone();
    let results = client::bounded(all.clone(), 4, move |g| {
        let ctx = ctx2.clone();
        async move { read_cell(&ctx, &g, cell).await }
    })
    .await;
    let mut daily = Vec::new();
    let mut past = Vec::new();
    for (i, (r, g)) in results.into_iter().zip(all.iter()).enumerate() {
        match r {
            Ok(v) if i < n_recent => daily.push(v),
            Ok(v) => past.push(v),
            Err(e) => gaps.push(format!("{}: {e}", g.title)),
        }
    }
    let latest = daily.iter().rev().find(|r| r.granule == latest_g.title).cloned();
    if latest.is_none() { gaps.push("latest SMAP granule could not be read".into()); }

    let comparison = (!past.is_empty()).then(|| {
        let sm = mean(past.iter().filter_map(|r| r.surface_m3m3));
        let rm = mean(past.iter().filter_map(|r| r.rootzone_m3m3));
        let l = latest.as_ref();
        SameDateComparison {
            years: past.iter().filter_map(|r| r.time.get(0..4)?.parse().ok()).collect(),
            surface_mean_m3m3: sm.map(r4),
            rootzone_mean_m3m3: rm.map(r4),
            surface_anomaly_m3m3: l.and_then(|l| Some(r4(l.surface_m3m3? - sm?))),
            rootzone_anomaly_m3m3: l.and_then(|l| Some(r4(l.rootzone_m3m3? - rm?))),
        }
    });

    Ok(SmapReport {
        provenance: Provenance::smap(),
        requested: common::geo::LatLon { lat, lon },
        cell,
        units: "m³/m³ volumetric water content".into(),
        latest,
        daily,
        same_date_past_years: past,
        comparison,
        note: "SMAP L4 is a 9 km model-assimilation product: it describes the surrounding landscape, not an irrigated field. \
               Granule times are UTC; SMAP L4 is published about 3-4 days after observation."
            .into(),
        gaps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_granule_time() {
        let t = granule_time("SMAP_L4_SM_gph_20261110T223000_Vv8013_001.h5_u9loH5HD").unwrap();
        assert_eq!(t.to_string(), "2026-11-10 22:30:00");
    }
}
