//! IMERG Late daily (GPM_3IMERGDL v07) rain at a point for the last N days: one GES DISC OPeNDAP read per day
//! (variable order [time][lon][lat]), at most 4 in flight, each day cached.
use crate::{client::{self, valid, CmrGranule}, ctx::Ctx, grid, meta::Provenance};
use anyhow::{bail, Context};
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SHORT_NAME: &str = "GPM_3IMERGDL";
const VERSION: &str = "07";
const OPENDAP: &str = "https://gpm1.gesdisc.eosdis.nasa.gov/opendap/GPM_L3/GPM_3IMERGDL.07";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RainDay {
    pub date: NaiveDate,
    pub granule: String,
    pub rain_mm: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImergCell {
    pub lon_index: usize,
    pub lat_index: usize,
    pub center_lat: f64,
    pub center_lon: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RainSummary {
    pub first_date: Option<NaiveDate>,
    pub last_date: Option<NaiveDate>,
    pub days_with_data: usize,
    pub total_mm: f64,
    /// Days with at least 1 mm.
    pub rainy_days: usize,
    pub wettest_day: Option<RainDay>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImergReport {
    pub provenance: Provenance,
    pub requested: common::geo::LatLon,
    pub cell: ImergCell,
    pub units: String,
    pub days_requested: u32,
    pub summary: RainSummary,
    /// Oldest first.
    pub daily: Vec<RainDay>,
    pub note: String,
    pub gaps: Vec<String>,
}

pub fn granule_url(g: &CmrGranule) -> anyhow::Result<String> {
    let file = g.producer_granule_id.clone()
        .or_else(|| g.title.split_once(':').map(|(_, f)| f.to_string()))
        .context("IMERG granule without a file name")?;
    let date = NaiveDate::parse_from_str(g.time_start.get(0..10).unwrap_or(""), "%Y-%m-%d").context("IMERG granule date")?;
    Ok(format!("{OPENDAP}/{}/{file}", date.format("%Y/%m")))
}

/// Daily rain (mm) of one 0.1° cell from one daily granule (2×2 block, the cell is the first element).
pub async fn read_day(ctx: &Ctx, g: &CmrGranule, i: usize, j: usize) -> anyhow::Result<RainDay> {
    let i0 = i.min(3598);
    let j0 = j.min(1798);
    let (di, dj) = (i - i0, j - j0);
    let ce = format!("/precipitation[0:0][{i0}:{}][{j0}:{}]", i0 + 1, j0 + 1);
    let url = granule_url(g)?;
    let date = NaiveDate::parse_from_str(&g.time_start[0..10], "%Y-%m-%d")?;
    let name = g.producer_granule_id.clone().unwrap_or_else(|| g.title.clone());
    let key = format!("earthdata:imerg:{name}:{i}:{j}");
    let (http, token) = (ctx.http.clone(), ctx.token.clone());
    ctx.cached("imerg", &key, 24 * 30, || async move {
        let d = client::dap_csv(&http, &token, &url, &ce, "IMERG").await?;
        let v = d.at3("/precipitation", 0, di, dj);
        if v.is_none() { bail!("IMERG returned no value for this cell"); }
        Ok(RainDay { date, granule: name, rain_mm: valid(v).map(|x| (x * 100.0).round() / 100.0) })
    })
    .await
}

pub fn summarize(daily: &[RainDay]) -> RainSummary {
    let with: Vec<&RainDay> = daily.iter().filter(|d| d.rain_mm.is_some()).collect();
    let total: f64 = with.iter().filter_map(|d| d.rain_mm).sum();
    RainSummary {
        first_date: daily.first().map(|d| d.date),
        last_date: daily.last().map(|d| d.date),
        days_with_data: with.len(),
        total_mm: (total * 10.0).round() / 10.0,
        rainy_days: with.iter().filter(|d| d.rain_mm.unwrap_or(0.0) >= 1.0).count(),
        wettest_day: with.iter().max_by(|a, b| a.rain_mm.partial_cmp(&b.rain_mm).unwrap()).map(|d| (*d).clone()),
    }
}

/// IMERG report for a point; `days` is clamped to 1..=90.
pub async fn report(ctx: &Ctx, lat: f64, lon: f64, days: u32) -> anyhow::Result<ImergReport> {
    let days = days.clamp(1, 90);
    let (i, j) = grid::imerg_index(lat, lon).context("point outside the IMERG grid")?;
    let (clat, clon) = grid::imerg_center(i, j);
    let today = Utc::now().date_naive();
    let start = today - Duration::days(days as i64 + 3);
    let temporal = vec![format!("{start}T00:00:00Z,{today}T23:59:59Z")];
    let granules = client::cmr_granules(&ctx.http, SHORT_NAME, VERSION, &temporal).await?;
    // One granule per date (CMR may list reprocessed duplicates; keep the last listed).
    let mut by_date: BTreeMap<String, CmrGranule> = BTreeMap::new();
    for g in granules {
        by_date.insert(g.time_start.get(0..10).unwrap_or("").to_string(), g);
    }
    let chosen: Vec<CmrGranule> = by_date.into_values().rev().take(days as usize).collect::<Vec<_>>().into_iter().rev().collect();
    if chosen.is_empty() { bail!("no IMERG Late daily granules found in CMR for the last {days} days"); }

    let ctx2 = ctx.clone();
    let results = client::bounded(chosen.clone(), 4, move |g| {
        let ctx = ctx2.clone();
        async move { read_day(&ctx, &g, i, j).await }
    })
    .await;
    let mut daily = Vec::new();
    let mut gaps = Vec::new();
    for (r, g) in results.into_iter().zip(chosen.iter()) {
        match r {
            Ok(d) => daily.push(d),
            Err(e) => gaps.push(format!("{}: {e}", &g.time_start[0..10])),
        }
    }
    if daily.len() < days as usize {
        gaps.push(format!("{} of {days} requested days available (IMERG Late runs ~1 day behind)", daily.len()));
    }
    Ok(ImergReport {
        provenance: Provenance::imerg(),
        requested: common::geo::LatLon { lat, lon },
        cell: ImergCell { lon_index: i, lat_index: j, center_lat: clat, center_lon: clon },
        units: "mm/day".into(),
        days_requested: days,
        summary: summarize(&daily),
        daily,
        note: "IMERG Late is a near-real-time satellite estimate for a ~10 km cell; light coastal drizzle is often missed. \
               Dates are UTC days."
            .into(),
        gaps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_gesdisc_url() {
        let g = CmrGranule {
            title: "GPM_3IMERGDL.07:3B-DAY-L.MS.MRG.3IMERG.20261112-S000000-E235959.V07C.nc4".into(),
            producer_granule_id: Some("3B-DAY-L.MS.MRG.3IMERG.20261112-S000000-E235959.V07C.nc4".into()),
            time_start: "2026-11-12T00:00:00.000Z".into(),
            collection_concept_id: None,
        };
        assert_eq!(
            granule_url(&g).unwrap(),
            "https://gpm1.gesdisc.eosdis.nasa.gov/opendap/GPM_L3/GPM_3IMERGDL.07/2026/11/3B-DAY-L.MS.MRG.3IMERG.20261112-S000000-E235959.V07C.nc4"
        );
    }

    #[test]
    fn summarizes_rain() {
        let d = |s: &str, v: Option<f64>| RainDay { date: s.parse().unwrap(), granule: String::new(), rain_mm: v };
        let s = summarize(&[d("2026-11-01", Some(0.2)), d("2026-11-02", Some(3.0)), d("2026-11-03", None)]);
        assert_eq!((s.days_with_data, s.rainy_days, s.total_mm), (2, 1, 3.2));
        assert_eq!(s.wettest_day.unwrap().rain_mm, Some(3.0));
    }

    #[test]
    fn huaral_constraint_matches_research_note() {
        let (i, j) = grid::imerg_index(-11.47, -77.21).unwrap();
        assert_eq!(format!("[{i}:{}][{j}:{}]", i + 1, j + 1), "[1027:1028][785:786]");
    }
}
