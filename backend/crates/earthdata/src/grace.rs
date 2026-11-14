//! GRACE/GRACE-FO mascon water-storage series. Served from pre-extracted JSON per example site
//! (`backend/data/grace_<site>.json`, made with `tools/grace_point.py`); other points get a clear gap.
//! The netCDF has no cloud OPeNDAP (DMR++) endpoint, so a live point read is not possible without the 46 MB file.
use crate::{grid::distance_km, meta::Provenance};
use anyhow::Context;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Points within this distance of an example site use that site's series (flagged).
pub const MATCH_KM: f64 = 50.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GracePoint {
    pub date: NaiveDate,
    pub lwe_cm: Option<f64>,
}

/// On-disk format of `backend/data/grace_<site>.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraceFile {
    pub site: Site,
    pub cell_center: common::geo::LatLon,
    pub granule: String,
    pub units: String,
    pub baseline: String,
    pub series: Vec<GracePoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraceStats {
    pub months: usize,
    pub first_date: NaiveDate,
    pub last_date: NaiveDate,
    pub min_cm: f64,
    pub max_cm: f64,
    pub latest_cm: f64,
    /// Ordinary least-squares slope over the whole record.
    pub trend_cm_per_year: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraceReport {
    pub provenance: Provenance,
    pub requested: common::geo::LatLon,
    pub available: bool,
    pub gap: Option<String>,
    pub example_site: Option<Site>,
    pub distance_km: Option<f64>,
    /// True when the series belongs to a nearby example site rather than being read for this exact point.
    pub nearest_example_site: bool,
    pub cell_center: Option<common::geo::LatLon>,
    pub granule: Option<String>,
    pub units: String,
    pub baseline: Option<String>,
    pub stats: Option<GraceStats>,
    pub series: Vec<GracePoint>,
    pub note: String,
}

fn decimal_year(d: NaiveDate) -> f64 {
    let days = if NaiveDate::from_ymd_opt(d.year(), 12, 31).unwrap().ordinal() == 366 { 366.0 } else { 365.0 };
    d.year() as f64 + (d.ordinal0() as f64) / days
}

/// Least-squares slope of value vs decimal year.
pub fn trend_per_year(points: &[(NaiveDate, f64)]) -> Option<f64> {
    if points.len() < 3 { return None; }
    let n = points.len() as f64;
    let xs: Vec<f64> = points.iter().map(|(d, _)| decimal_year(*d)).collect();
    let mx = xs.iter().sum::<f64>() / n;
    let my = points.iter().map(|p| p.1).sum::<f64>() / n;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (x, (_, y)) in xs.iter().zip(points) {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
    }
    (sxx > 0.0).then(|| sxy / sxx)
}

pub fn stats(series: &[GracePoint]) -> Option<GraceStats> {
    let pts: Vec<(NaiveDate, f64)> = series.iter().filter_map(|p| Some((p.date, p.lwe_cm?))).collect();
    let (first, last) = (pts.first()?, pts.last()?);
    let r = |x: f64| (x * 1000.0).round() / 1000.0;
    Some(GraceStats {
        months: pts.len(),
        first_date: first.0,
        last_date: last.0,
        min_cm: pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
        max_cm: pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
        latest_cm: last.1,
        trend_cm_per_year: r(trend_per_year(&pts)?),
    })
}

/// Loads every `grace_*.json` in `dir`.
pub fn load_sites(dir: &Path) -> anyhow::Result<Vec<GraceFile>> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return Ok(out) };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("grace_") && name.ends_with(".json") {
            let text = std::fs::read_to_string(e.path())?;
            out.push(serde_json::from_str(&text).with_context(|| format!("bad GRACE data file {name}"))?);
        }
    }
    Ok(out)
}

/// GRACE report for a point from the example-site files in `dir`.
pub fn report_from_dir(dir: &Path, lat: f64, lon: f64) -> anyhow::Result<GraceReport> {
    let sites = load_sites(dir)?;
    let nearest = sites.into_iter()
        .map(|f| (distance_km(lat, lon, f.site.lat, f.site.lon), f))
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let base = GraceReport {
        provenance: Provenance::grace(),
        requested: common::geo::LatLon { lat, lon },
        available: false,
        gap: None,
        example_site: None,
        distance_km: None,
        nearest_example_site: false,
        cell_center: None,
        granule: None,
        units: "cm liquid water equivalent thickness".into(),
        baseline: None,
        stats: None,
        series: vec![],
        note: "GRACE senses total water storage (soil, groundwater, snow, surface water) over ~300 km: regional context for \
               long-term wetting or drying, never a reading for one field."
            .into(),
    };
    match nearest {
        Some((d, f)) if d <= MATCH_KM => Ok(GraceReport {
            available: true,
            distance_km: Some((d * 10.0).round() / 10.0),
            nearest_example_site: d > 1.0,
            stats: stats(&f.series),
            cell_center: Some(f.cell_center),
            granule: Some(f.granule),
            baseline: Some(f.baseline),
            units: f.units,
            series: f.series,
            example_site: Some(f.site),
            ..base
        }),
        _ => Ok(GraceReport {
            gap: Some(format!(
                "GRACE regional series available for example sites only (none within {MATCH_KM} km of this point)."
            )),
            ..base
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slope_of_a_line() {
        let pts: Vec<(NaiveDate, f64)> = (0..10)
            .map(|y| (NaiveDate::from_ymd_opt(2010 + y, 1, 1).unwrap(), 2.0 * y as f64 + 1.0))
            .collect();
        assert!((trend_per_year(&pts).unwrap() - 2.0).abs() < 1e-3);
    }

    #[test]
    fn huaral_file_is_served_and_far_points_get_a_gap() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let r = report_from_dir(&dir, -11.47, -77.21).unwrap();
        assert!(r.available, "{:?}", r.gap);
        let s = r.stats.unwrap();
        assert_eq!(s.months, 261);
        assert!((s.trend_cm_per_year - 0.24).abs() < 0.03, "{}", s.trend_cm_per_year);
        let far = report_from_dir(&dir, 40.0, -100.0).unwrap();
        assert!(!far.available && far.gap.is_some());
    }
}
