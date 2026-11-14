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

/// A real irrigated plot (~5 ha) in the lower Huaral (Chancay) valley, Lima region, Peru, traced by hand on the
/// Sentinel-2 L2A scene of 2026-11-13 (Contains modified Copernicus Sentinel data 2026). Centre about 11.473 S, 77.206 W.
pub fn huaral_ring() -> Vec<[f64; 2]> {
    vec![
        [-77.207403, -11.471998], [-77.206417, -11.471659], [-77.204745, -11.472721],
        [-77.205638, -11.473648], [-77.206692, -11.474191], [-77.207426, -11.473287],
    ]
}

pub fn all() -> Vec<ExampleSite> {
    let ring = huaral_ring();
    let field = Field { center: LatLon { lat: -11.4729, lon: -77.2061 }, ring };
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
    fn huaral_is_about_five_hectares() {
        let s = super::get("huaral").unwrap();
        assert!((4.5..5.6).contains(&s.field.area_ha), "{}", s.field.area_ha);
    }
}
