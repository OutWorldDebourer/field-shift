//! Small geometry helpers shared by services: points, field outlines, areas, bounding boxes and WKT.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

/// A field: a centre point, plus an optional ring of [lon, lat] vertices (GeoJSON order, not closed).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub center: LatLon,
    #[serde(default)]
    pub ring: Vec<[f64; 2]>,
}

impl Field {
    pub fn point(lat: f64, lon: f64) -> Self { Self { center: LatLon { lat, lon }, ring: vec![] } }

    /// Area in hectares from the ring (spherical polygon area, fine for small fields). 0 for a point.
    pub fn area_ha(&self) -> f64 {
        if self.ring.len() < 3 { return 0.0; }
        let r = 6_371_008.8_f64;
        let n = self.ring.len();
        let mut s = 0.0;
        for i in 0..n {
            let [x1, y1] = self.ring[i];
            let [x2, y2] = self.ring[(i + 1) % n];
            s += (x2 - x1).to_radians() * (2.0 + y1.to_radians().sin() + y2.to_radians().sin());
        }
        (s * r * r / 2.0).abs() / 10_000.0
    }

    /// [west, south, east, north]; a point gets a box of `pad_deg` around it.
    pub fn bbox(&self, pad_deg: f64) -> [f64; 4] {
        if self.ring.is_empty() {
            let c = self.center;
            return [c.lon - pad_deg, c.lat - pad_deg, c.lon + pad_deg, c.lat + pad_deg];
        }
        let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        for [x, y] in &self.ring {
            b[0] = b[0].min(*x);
            b[1] = b[1].min(*y);
            b[2] = b[2].max(*x);
            b[3] = b[3].max(*y);
        }
        b
    }

    /// True if (lon, lat) is inside the ring (ray casting). A point field contains nothing.
    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        let n = self.ring.len();
        if n < 3 { return false; }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let [xi, yi] = self.ring[i];
            let [xj, yj] = self.ring[j];
            if (yi > lat) != (yj > lat) && lon < (xj - xi) * (lat - yi) / (yj - yi) + xi {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// WKT for MariaDB `ST_GeomFromText`. A point field becomes a ~100 m square so the POLYGON column always holds a value.
    pub fn wkt(&self) -> String {
        let ring: Vec<[f64; 2]> = if self.ring.len() >= 3 {
            self.ring.clone()
        } else {
            let [w, s, e, n] = self.bbox(0.0005);
            vec![[w, s], [e, s], [e, n], [w, n]]
        };
        let mut pts: Vec<String> = ring.iter().map(|[x, y]| format!("{x} {y}")).collect();
        pts.push(format!("{} {}", ring[0][0], ring[0][1]));
        format!("POLYGON(({}))", pts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_of_100m_square_is_about_1ha() {
        let d = 100.0 / 111_320.0;
        let f = Field { center: LatLon { lat: 0.0, lon: 0.0 }, ring: vec![[0.0, 0.0], [d, 0.0], [d, d], [0.0, d]] };
        assert!((f.area_ha() - 1.0).abs() < 0.02, "{}", f.area_ha());
        assert!(f.contains(d / 2.0, d / 2.0));
        assert!(!f.contains(2.0 * d, d / 2.0));
    }
}
