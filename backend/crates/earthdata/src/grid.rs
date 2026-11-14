//! Grid indexing: EASE-Grid 2.0 global 9 km (SMAP L4) and the IMERG 0.1° grid.
//!
//! EASE-Grid 2.0 global (EPSG:6933) is a Lambert cylindrical equal-area projection on the WGS 84 ellipsoid with
//! standard parallels ±30° (Brodzik et al. 2012, doi:10.3390/ijgi1010032):
//!   k0 = cos(30°) / sqrt(1 - e² sin²30°)
//!   x  = a k0 (λ - λ0),  λ0 = 0
//!   y  = a q(φ) / (2 k0),  q(φ) = (1 - e²) [ sinφ / (1 - e² sin²φ) - 1/(2e) ln((1 - e sinφ) / (1 + e sinφ)) ]
//! The 9 km grid is 3856 columns × 1624 rows of 9008.055210146 m, upper-left corner at
//! x0 = -17367530.44516138 m, y0 = 7314540.830638 m. Row 0 is the north edge:
//!   col = floor((x - x0) / cell),  row = floor((y0 - y) / cell)
//! Verified against SMAP L4 `cell_lat` / `cell_lon` (row 974, col 1102 → -11.5503, -77.0695).
use serde::{Deserialize, Serialize};

const A: f64 = 6_378_137.0;
const E2: f64 = 0.006_694_379_990_141_316_5;
const CELL_M: f64 = 9_008.055_210_146;
const X0: f64 = -17_367_530.445_161_38;
const Y0: f64 = 7_314_540.830_638;
pub const EASE_COLS: usize = 3856;
pub const EASE_ROWS: usize = 1624;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridCell {
    pub row: usize,
    pub col: usize,
    pub center_lat: f64,
    pub center_lon: f64,
}

fn k0() -> f64 {
    let s = 30f64.to_radians().sin();
    30f64.to_radians().cos() / (1.0 - E2 * s * s).sqrt()
}

fn q(phi: f64) -> f64 {
    let e = E2.sqrt();
    let s = phi.sin();
    (1.0 - E2) * (s / (1.0 - E2 * s * s) - (1.0 / (2.0 * e)) * ((1.0 - e * s) / (1.0 + e * s)).ln())
}

/// Centre of an EASE-Grid 2.0 9 km cell (inverse projection, authalic-latitude series).
pub fn ease9_center(row: usize, col: usize) -> (f64, f64) {
    let k0 = k0();
    let x = X0 + (col as f64 + 0.5) * CELL_M;
    let y = Y0 - (row as f64 + 0.5) * CELL_M;
    let lon = (x / (A * k0)).to_degrees();
    let beta = (2.0 * y * k0 / A / q(std::f64::consts::FRAC_PI_2)).clamp(-1.0, 1.0).asin();
    let lat = beta
        + (E2 / 3.0 + 31.0 * E2 * E2 / 180.0) * (2.0 * beta).sin()
        + (517.0 * E2 * E2 / 5040.0) * (4.0 * beta).sin();
    (lat.to_degrees(), lon)
}

/// The EASE-Grid 2.0 9 km cell containing (lat, lon), or None outside the grid (|lat| > ~85°).
pub fn ease9_cell(lat: f64, lon: f64) -> Option<GridCell> {
    let k0 = k0();
    let x = A * k0 * lon.to_radians();
    let y = A * q(lat.to_radians()) / (2.0 * k0);
    let c = ((x - X0) / CELL_M).floor();
    let r = ((Y0 - y) / CELL_M).floor();
    if !(0.0..EASE_COLS as f64).contains(&c) || !(0.0..EASE_ROWS as f64).contains(&r) {
        return None;
    }
    let (row, col) = (r as usize, c as usize);
    let (center_lat, center_lon) = ease9_center(row, col);
    Some(GridCell { row, col, center_lat, center_lon })
}

/// IMERG 0.1° grid: lon index 0..3600 from -180, lat index 0..1800 from -90; centres at -179.95 + 0.1 i, -89.95 + 0.1 j.
/// Returned as (lon_index, lat_index) because the variable is ordered [time][lon][lat].
pub fn imerg_index(lat: f64, lon: f64) -> Option<(usize, usize)> {
    if !(-90.0..90.0).contains(&lat) || !(-180.0..180.0).contains(&lon) {
        return None;
    }
    let i = (((lon + 180.0) / 0.1).floor() as usize).min(3599);
    let j = (((lat + 90.0) / 0.1).floor() as usize).min(1799);
    Some((i, j))
}

pub fn imerg_center(i: usize, j: usize) -> (f64, f64) {
    (-89.95 + 0.1 * j as f64, -179.95 + 0.1 * i as f64)
}

/// Great-circle distance in km.
pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dp = p2 - p1;
    let dl = (lon2 - lon1).to_radians();
    let h = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * 6371.0 * h.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_cell_from_research_note_matches_smap_cell_lat_lon() {
        // The research note's OPeNDAP read returned cell centre -11.55, -77.07 for row 974, col 1102.
        let (lat, lon) = ease9_center(974, 1102);
        assert!((lat - -11.5503).abs() < 1e-3, "{lat}");
        assert!((lon - -77.0695).abs() < 1e-3, "{lon}");
    }

    #[test]
    fn ease_huaral_point_falls_in_its_own_cell() {
        // Verified against SMAP cell_lat/cell_lon: row 973 centre -11.4783, col 1100 centre -77.2562.
        let c = ease9_cell(-11.47, -77.21).unwrap();
        assert_eq!((c.row, c.col), (973, 1100));
        assert!((c.center_lat - -11.4783).abs() < 1e-3);
        assert!((c.center_lon - -77.2562).abs() < 1e-3);
        // The point lies within half a cell (~0.047° lon, ~0.036° lat here) of the centre.
        assert!((c.center_lat - -11.47).abs() < 0.04 && (c.center_lon - -77.21).abs() < 0.05);
    }

    #[test]
    fn ease_round_trip_and_grid_centre() {
        let c = ease9_cell(0.01, 0.01).unwrap();
        assert_eq!((c.row, c.col), (EASE_ROWS / 2 - 1, EASE_COLS / 2));
        for (r, cc) in [(10, 10), (800, 1900), (1600, 3800)] {
            let (lat, lon) = ease9_center(r, cc);
            let back = ease9_cell(lat, lon).unwrap();
            assert_eq!((back.row, back.col), (r, cc));
        }
        assert!(ease9_cell(89.0, 0.0).is_none());
    }

    #[test]
    fn imerg_huaral_index() {
        let (i, j) = imerg_index(-11.47, -77.21).unwrap();
        assert_eq!((i, j), (1027, 785));
        let (lat, lon) = imerg_center(i, j);
        assert!((lat - -11.45).abs() < 1e-9 && (lon - -77.25).abs() < 1e-9);
    }
}
