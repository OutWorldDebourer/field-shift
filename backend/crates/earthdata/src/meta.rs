//! Provenance attached to every Earthdata value: source, dataset id + version, DOI, resolution, badge and license.
use serde::{Deserialize, Serialize};

pub const BADGE: &str = "regional context";
pub const LICENSE: &str = "NASA Earth science data, no restrictions on use; cite the dataset DOI";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Provenance {
    pub source: String,
    pub dataset_id: String,
    pub version: String,
    pub doi: String,
    pub doi_url: String,
    pub resolution: String,
    pub badge: String,
    pub license: String,
}

impl Provenance {
    fn new(source: &str, dataset_id: &str, version: &str, doi: &str, resolution: &str) -> Self {
        Self {
            source: source.into(),
            dataset_id: dataset_id.into(),
            version: version.into(),
            doi: doi.into(),
            doi_url: format!("https://doi.org/{doi}"),
            resolution: resolution.into(),
            badge: BADGE.into(),
            license: LICENSE.into(),
        }
    }

    pub fn smap() -> Self {
        Self::new(
            "NASA SMAP L4 Global 3-hourly 9 km EASE-Grid Surface and Root Zone Soil Moisture Geophysical Data (NSIDC DAAC)",
            "SPL4SMGP",
            "008",
            "10.5067/T5RUATAQREF8",
            "9 km EASE-Grid 2.0 cell (model-assimilated, not the irrigated field itself)",
        )
    }

    pub fn imerg() -> Self {
        Self::new(
            "NASA GPM IMERG Late Precipitation L3 1 day 0.1° (GES DISC)",
            "GPM_3IMERGDL",
            "07",
            "10.5067/GPM/IMERGDL/DAY/07",
            "0.1° grid cell (~10 km)",
        )
    }

    pub fn grace() -> Self {
        Self::new(
            "NASA JPL GRACE/GRACE-FO Mascon CRI RL06.3 v4 (PO.DAAC)",
            "TELLUS_GRAC-GRFO_MASCON_CRI_GRID_RL06.3_V4",
            "RL06.3Mv04",
            "10.5067/TEMSC-3JC634",
            "~300 km mascon (0.5° grid posting)",
        )
    }
}
