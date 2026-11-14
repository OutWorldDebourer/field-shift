//! NEX-GDDP-CMIP6 projection for a point, served from precomputed files `backend/data/projection_<site>.json`
//! (made with `tools/nex_gddp_point.py`). Points without a file within `MATCH_KM` get a clear gap.
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const MATCH_KM: f64 = 50.0;
pub const BADGE: &str = "regional context";
pub const LABEL: &str = "projection, not a forecast";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site { pub id: String, pub name: String, pub lat: f64, pub lon: f64 }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stat { pub median: f64, pub min: f64, pub max: f64, pub n: usize }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Baseline { pub tas_c: Stat, pub pr_mm_year: Stat }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioChange {
    pub tas_change_c: Stat,
    pub pr_change_mm_year: Stat,
    pub pr_change_pct: Option<Stat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ensemble {
    pub baseline: Baseline,
    #[serde(flatten)]
    pub scenarios: BTreeMap<String, ScenarioChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelEntry {
    pub model: String,
    pub variant: String,
    pub file_license: Option<String>,
    pub file_version: Option<String>,
    pub upstream_cmip6_license: String,
    pub baseline: serde_json::Value,
    pub scenarios: serde_json::Value,
}

/// On-disk format written by `tools/nex_gddp_point.py`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectionFile {
    pub site: Site,
    pub cell_center: Option<common::geo::LatLon>,
    pub dataset: String,
    pub dataset_version: String,
    pub doi: String,
    pub paper_doi: String,
    pub resolution: String,
    pub baseline_period: String,
    pub future_period: String,
    pub variables: serde_json::Value,
    pub ensemble: Ensemble,
    pub models: Vec<ModelEntry>,
    pub license: serde_json::Value,
    pub citation: String,
    pub acknowledgement: String,
    pub access: String,
    pub generated: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectionReport {
    pub requested: common::geo::LatLon,
    pub available: bool,
    pub gap: Option<String>,
    pub badge: String,
    pub label: String,
    pub example_site: Option<Site>,
    pub distance_km: Option<f64>,
    /// True when the projection belongs to a nearby example site rather than computed for this exact point.
    pub nearest_example_site: bool,
    pub projection: Option<ProjectionFile>,
    pub note: String,
}

fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let h = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * ((lon2 - lon1).to_radians() / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().asin()
}

/// Loads every `projection_*.json` in `dir`.
pub fn load(dir: &Path) -> anyhow::Result<Vec<ProjectionFile>> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return Ok(out) };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("projection_") && name.ends_with(".json") {
            let text = std::fs::read_to_string(e.path())?;
            out.push(serde_json::from_str(&text).with_context(|| format!("bad projection file {name}"))?);
        }
    }
    Ok(out)
}

pub fn report_from_dir(dir: &Path, lat: f64, lon: f64) -> anyhow::Result<ProjectionReport> {
    let nearest = load(dir)?.into_iter()
        .map(|f| (distance_km(lat, lon, f.site.lat, f.site.lon), f))
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut r = ProjectionReport {
        requested: common::geo::LatLon { lat, lon },
        available: false,
        gap: None,
        badge: BADGE.into(),
        label: LABEL.into(),
        example_site: None,
        distance_km: None,
        nearest_example_site: false,
        projection: None,
        note: "Downscaled climate-model ensemble (0.25°, ~25 km). It shows a plausible range of change under an emissions \
               scenario, not a forecast for a given year; models disagree, so read the range, not just the median."
            .into(),
    };
    match nearest {
        Some((d, f)) if d <= MATCH_KM => {
            r.available = true;
            r.distance_km = Some((d * 10.0).round() / 10.0);
            r.nearest_example_site = d > 1.0;
            r.example_site = Some(f.site.clone());
            r.projection = Some(f);
        }
        _ => r.gap = Some(format!(
            "Climate projection precomputed for example sites only (none within {MATCH_KM} km of this point)."
        )),
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }

    #[test]
    fn huaral_projection_is_served() {
        let r = report_from_dir(&data_dir(), -11.47, -77.21).unwrap();
        assert!(r.available, "{:?}", r.gap);
        assert!(!r.nearest_example_site);
        let p = r.projection.unwrap();
        let s = &p.ensemble.scenarios["ssp245"];
        assert!(s.tas_change_c.median > 0.0 && s.tas_change_c.median < 3.0, "{:?}", s.tas_change_c);
        assert!(s.tas_change_c.min <= s.tas_change_c.median && s.tas_change_c.median <= s.tas_change_c.max);
        assert!(p.models.len() >= 5);
    }

    #[test]
    fn nearby_point_is_flagged_and_far_point_gets_gap() {
        let near = report_from_dir(&data_dir(), -11.60, -77.10).unwrap();
        assert!(near.available && near.nearest_example_site);
        let far = report_from_dir(&data_dir(), 40.0, -100.0).unwrap();
        assert!(!far.available && far.gap.is_some());
    }
}
