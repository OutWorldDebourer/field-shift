//! Provenance attached to every value: source, dataset, period, resolution, scale badge, license and method.
use serde::Serialize;

pub const SOURCE: &str = "NASA POWER (MERRA-2 / CERES SYN1deg)";
pub const DATASET: &str = "POWER Daily API v2, community AG, https://power.larc.nasa.gov/api/temporal/daily/point";
pub const RESOLUTION: &str = "~0.5° × 0.625° grid cell (about 50 km)";
pub const BADGE: &str = "regional context";
pub const LICENSE: &str = "NASA open data, no restrictions; cite NASA POWER";

#[derive(Clone, Debug, Serialize)]
pub struct Meta {
    pub source: &'static str,
    pub dataset: &'static str,
    /// Dates the value covers, e.g. "2026-10-11..2026-11-09" or "1985-2024 same calendar window".
    pub period: String,
    pub resolution: &'static str,
    pub badge: &'static str,
    pub license: &'static str,
    pub method: String,
}

impl Meta {
    pub fn new(period: impl Into<String>, method: impl Into<String>) -> Self {
        Self { source: SOURCE, dataset: DATASET, period: period.into(), resolution: RESOLUTION, badge: BADGE, license: LICENSE, method: method.into() }
    }
}
