//! Sentinel-2 / Landsat NDVI and NASADEM terrain → field-level readings and the NDVI series.
use crate::reading::{bi, ndvi_verdict, round, Reading};
use serde_json::{json, Value};

pub fn readings(ndvi: Option<&Value>, terrain: Option<&Value>) -> Vec<Reading> {
    let mut out = vec![];
    if let Some(n) = ndvi {
        if let Some(l) = n.get("latest").filter(|l| !l.is_null()) {
            let v = l["mean"].as_f64().unwrap_or(0.0);
            let bare = n["bare_soil"].as_array().map(|b| b.len()).unwrap_or(0);
            let series = n["points"].as_array().into_iter().flatten().rev().take(40).collect::<Vec<_>>().into_iter().rev()
                .map(|p| json!({ "t": p["date"], "v": p["mean"].as_f64().map(|x| round(x, 3)) })).collect();
            out.push(Reading {
                id: "ndvi".into(),
                label: bi("Crop vigor (NDVI), latest clear image", "Vigor del cultivo (NDVI), última imagen despejada"),
                value: Some(round(v, 2)), unit: "NDVI".into(), verdict: ndvi_verdict(v), key: false,
                source: l["source"].as_str().unwrap_or("Sentinel-2 L2A").into(),
                dataset: l["scene_id"].as_str().unwrap_or("").into(),
                date: l["date"].as_str().unwrap_or("").into(),
                resolution: format!("{} m", l["resolution_m"].as_f64().unwrap_or(10.0)),
                badge: "field-level".into(),
                license: l["credit"].as_str().unwrap_or("Contains modified Copernicus Sentinel data").into(),
                method: Some(format!("Mean NDVI over the field outline, cloud-masked; {} clear observations in 24 months; {} bare-soil period(s) detected (NDVI < 0.2 twice in a row).",
                    n["points"].as_array().map(|p| p.len()).unwrap_or(0), bare)),
                series,
            });
        }
    }
    if let Some(t) = terrain {
        if let Some(s) = t["slope_mean_deg"].as_f64() {
            let pct = (s.to_radians().tan() * 100.0).round();
            out.push(Reading {
                id: "slope".into(),
                label: bi("Slope of the field", "Pendiente del campo"),
                value: Some(round(s, 1)), unit: "°".into(),
                verdict: if s < 2.0 { bi(format!("nearly flat (~{pct}%)"), format!("casi plano (~{pct}%)")) }
                    else if s < 6.0 { bi(format!("gentle slope (~{pct}%): watch furrow erosion"), format!("pendiente suave (~{pct}%): cuidar erosión en surcos")) }
                    else { bi(format!("steep (~{pct}%): erosion risk"), format!("pronunciada (~{pct}%): riesgo de erosión")) },
                key: false, source: "NASADEM".into(),
                dataset: t["scene_id"].as_str().unwrap_or("NASADEM_HGT").into(),
                date: t["date"].as_str().unwrap_or("2000-02-20").into(),
                resolution: "30 m".into(), badge: "field-level".into(),
                license: t["license"].as_str().unwrap_or("NASA, public domain").into(),
                method: t["method"].as_str().map(String::from),
                series: vec![],
            });
        }
    }
    out
}

pub fn series(ndvi: Option<&Value>) -> Vec<Value> {
    ndvi.and_then(|n| n["points"].as_array()).into_iter().flatten().map(|p| json!({
        "date": p["date"], "ndvi": p["mean"].as_f64().map(|x| round(x, 3)), "sensor": p["source"], "scene": p["scene_id"],
    })).collect()
}

/// Snapshot size: three times the field's diagonal, at least 600 m.
pub fn snapshot_size_m(field: &common::geo::Field) -> f64 {
    let [w, s, e, n] = field.bbox(0.0);
    if field.ring.len() < 3 { return 800.0; }
    let dx = (e - w) * 111_320.0 * field.center.lat.to_radians().cos();
    let dy = (n - s) * 110_574.0;
    ((dx * dx + dy * dy).sqrt() * 3.0).max(600.0).round()
}
