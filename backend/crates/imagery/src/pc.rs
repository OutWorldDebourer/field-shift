//! Thin client for Microsoft Planetary Computer (no account): STAC search and the data API (titiler) statistics.
use anyhow::{anyhow, bail, Context};
use common::geo::Field;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

pub const STAC: &str = "https://planetarycomputer.microsoft.com/api/stac/v1";
pub const DATA: &str = "https://planetarycomputer.microsoft.com/api/data/v1";

/// Half side (degrees) of the square used when a field is only a point: ~110 m box, same as `Field::wkt`.
pub const POINT_HALF_DEG: f64 = 0.0005;

/// Closed ring [lon, lat] of the field; a point field becomes a ~110 m square around its centre.
pub fn field_ring(field: &Field) -> Vec<[f64; 2]> {
    let mut ring = if field.ring.len() >= 3 {
        field.ring.clone()
    } else {
        let [w, s, e, n] = Field::point(field.center.lat, field.center.lon).bbox(POINT_HALF_DEG);
        vec![[w, s], [e, s], [e, n], [w, n]]
    };
    if ring.first() != ring.last() {
        ring.push(ring[0]);
    }
    ring
}

/// The field as a polygon `Field` (ring not closed), so `contains` works for point fields too.
pub fn field_as_polygon(field: &Field) -> Field {
    let mut ring = field_ring(field);
    ring.pop();
    Field { center: field.center, ring }
}

pub fn polygon_geometry(ring: &[[f64; 2]]) -> Value {
    json!({ "type": "Polygon", "coordinates": [ring] })
}

pub fn feature(ring: &[[f64; 2]]) -> Value {
    json!({ "type": "Feature", "properties": {}, "geometry": polygon_geometry(ring) })
}

/// Square box of `size_m` metres around a point: [west, south, east, north].
pub fn bbox_around(lat: f64, lon: f64, size_m: f64) -> [f64; 4] {
    let half_lat = size_m / 2.0 / 111_320.0;
    let half_lon = size_m / 2.0 / (111_320.0 * lat.to_radians().cos().max(0.01));
    [lon - half_lon, lat - half_lat, lon + half_lon, lat + half_lat]
}

/// Stable 64-bit FNV-1a hash (cache keys must not change between builds).
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Cache key fragment for a field: centre and ring rounded to ~1 m.
pub fn field_key(field: &Field) -> String {
    let mut s = format!("{:.5},{:.5}", field.center.lat, field.center.lon);
    for [x, y] in &field.ring {
        s.push_str(&format!(";{x:.5},{y:.5}"));
    }
    format!("{:016x}", fnv1a(&s))
}

/// Summary statistics of one band / expression over a feature.
#[derive(Debug, Clone, Default)]
pub struct BandStats {
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub p10: Option<f64>,
    pub p90: Option<f64>,
    pub valid_pixels: f64,
}

async fn send_with_retry(rb: reqwest::RequestBuilder) -> anyhow::Result<reqwest::Response> {
    let mut delay = Duration::from_millis(800);
    for attempt in 0..3 {
        let req = rb.try_clone().ok_or_else(|| anyhow!("request not clonable"))?;
        match req.send().await {
            Ok(r) if r.status().as_u16() == 429 || r.status().is_server_error() => {
                if attempt == 2 {
                    bail!("Planetary Computer returned HTTP {}", r.status());
                }
            }
            Ok(r) => return Ok(r),
            Err(e) if attempt == 2 => return Err(anyhow!("Planetary Computer request failed: {}", e.without_url())),
            Err(_) => {}
        }
        tokio::time::sleep(delay).await;
        delay *= 3;
    }
    unreachable!()
}

/// POST /item/statistics with a `;`-separated band-math `expression`; returns one entry per sub-expression, in order.
pub async fn statistics(
    http: &reqwest::Client,
    collection: &str,
    item: &str,
    expression: &str,
    feature: &Value,
    percentiles: bool,
) -> anyhow::Result<Vec<BandStats>> {
    let mut q: Vec<(&str, &str)> = vec![
        ("collection", collection),
        ("item", item),
        ("expression", expression),
        ("asset_as_band", "true"),
    ];
    if percentiles {
        q.push(("p", "10"));
        q.push(("p", "90"));
    }
    let resp = send_with_retry(http.post(format!("{DATA}/item/statistics")).query(&q).json(feature)).await?;
    let status = resp.status();
    let body: Value = resp.json().await.map_err(|e| anyhow!("statistics: bad JSON: {}", e.without_url()))?;
    if !status.is_success() {
        bail!("statistics HTTP {status}: {}", body.get("detail").map(|d| d.to_string()).unwrap_or_default());
    }
    let stats: BTreeMap<String, Value> = serde_json::from_value(body["properties"]["statistics"].clone())
        .context("statistics: missing properties.statistics")?;
    expression
        .split(';')
        .enumerate()
        .map(|(i, sub)| {
            let v = stats
                .get(sub)
                .or_else(|| stats.get(&format!("b{}", i + 1)))
                .ok_or_else(|| anyhow!("statistics: no result for band {}", i + 1))?;
            let f = |k: &str| v.get(k).and_then(Value::as_f64);
            Ok(BandStats {
                mean: f("mean").unwrap_or(f64::NAN),
                min: f("min").unwrap_or(f64::NAN),
                max: f("max").unwrap_or(f64::NAN),
                p10: f("percentile_10"),
                p90: f("percentile_90"),
                valid_pixels: f("valid_pixels").or_else(|| f("count")).unwrap_or(0.0),
            })
        })
        .collect()
}

/// GET a rendered or raw image from the data API (`/item/bbox/...`); returns the bytes.
pub async fn get_bytes(http: &reqwest::Client, url: &str, query: &[(&str, &str)]) -> anyhow::Result<Vec<u8>> {
    let resp = send_with_retry(http.get(url).query(query)).await?;
    let status = resp.status();
    let bytes = resp.bytes().await.map_err(|e| anyhow!("data API read failed: {}", e.without_url()))?;
    if !status.is_success() {
        bail!("data API HTTP {status}: {}", String::from_utf8_lossy(&bytes[..bytes.len().min(300)]));
    }
    Ok(bytes.to_vec())
}

/// POST to the STAC search endpoint, following `next` links (at most `max_pages`).
pub async fn stac_search(http: &reqwest::Client, body: Value, max_pages: usize) -> anyhow::Result<Vec<Value>> {
    let mut out = Vec::new();
    let mut next: Option<(String, Option<Value>)> = Some((format!("{STAC}/search"), Some(body)));
    let mut pages = 0;
    while let Some((href, body)) = next.take() {
        let rb = match &body {
            Some(b) => http.post(&href).json(b),
            None => http.get(&href),
        };
        let resp = send_with_retry(rb).await?;
        let status = resp.status();
        let page: Value = resp.json().await.map_err(|e| anyhow!("STAC: bad JSON: {}", e.without_url()))?;
        if !status.is_success() {
            bail!("STAC search HTTP {status}");
        }
        if let Some(fs) = page["features"].as_array() {
            out.extend(fs.iter().cloned());
        }
        pages += 1;
        if pages >= max_pages {
            break;
        }
        next = page["links"].as_array().and_then(|ls| {
            ls.iter().find(|l| l["rel"] == "next").and_then(|l| {
                let href = l["href"].as_str()?.to_string();
                let body = if l["method"] == "POST" { Some(l["body"].clone()) } else { None };
                Some((href, body))
            })
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_field_ring_is_closed_square() {
        let r = field_ring(&Field::point(-11.47, -77.21));
        assert_eq!(r.len(), 5);
        assert_eq!(r[0], r[4]);
        assert!((r[1][0] - r[0][0] - 2.0 * POINT_HALF_DEG).abs() < 1e-12);
    }

    #[test]
    fn field_key_is_stable() {
        assert_eq!(fnv1a(""), 0xcbf29ce484222325);
        assert_eq!(field_key(&Field::point(-11.47, -77.21)), field_key(&Field::point(-11.470001, -77.210001)));
    }
}
