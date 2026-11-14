//! Example sites judges can open without a GPS fix. Huaral is the pilot (precomputed and cached).
use crate::model::{Bi, FieldOut};
use common::geo::{Field, LatLon};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ExampleSite {
    pub id: &'static str,
    pub name: Bi,
    pub field: FieldOut,
    /// Crop story used by the scripted demo mode.
    pub crop_now: &'static str,
    pub crop_before: &'static str,
}

/// A ~2.1 ha irrigated plot outline in the lower Huaral (Chancay) valley, Lima region, Peru.
/// Drawn as a 150 m x 140 m quadrilateral around the pilot point used in research/01_api_tests.md.
pub fn huaral_ring() -> Vec<[f64; 2]> {
    let (lat, lon) = (-11.47_f64, -77.21_f64);
    let dlat = 70.0 / 110_574.0;
    let dlon = 75.0 / (111_320.0 * lat.to_radians().cos());
    vec![[lon - dlon, lat - dlat], [lon + dlon, lat - dlat * 0.92], [lon + dlon * 0.96, lat + dlat], [lon - dlon, lat + dlat * 1.02]]
}

pub fn all() -> Vec<ExampleSite> {
    let ring = huaral_ring();
    let field = Field { center: LatLon { lat: -11.47, lon: -77.21 }, ring };
    vec![ExampleSite {
        id: "huaral",
        name: Bi::new("Huaral valley, Peru", "Valle de Huaral, Perú"),
        field: FieldOut { area_ha: (field.area_ha() * 10.0).round() / 10.0, center: field.center, ring: field.ring, label: "Huaral valley, Peru".into() },
        crop_now: "pea",
        crop_before: "lettuce",
    }]
}

pub fn get(id: &str) -> Option<ExampleSite> { all().into_iter().find(|s| s.id == id) }

#[cfg(test)]
mod tests {
    #[test]
    fn huaral_is_about_two_hectares() {
        let s = super::get("huaral").unwrap();
        assert!((1.9..2.3).contains(&s.field.area_ha), "{}", s.field.area_ha);
    }
}
