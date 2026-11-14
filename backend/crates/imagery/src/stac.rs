//! STAC scene search for a field: Sentinel-2 L2A and Landsat Collection 2 Level-2 (Landsat 8 and 9).
use crate::pc;
use chrono::NaiveDate;
use common::geo::Field;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const BADGE: &str = "field-level";
pub const PC_CREDIT: &str = "processed by Microsoft Planetary Computer";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Collection {
    Sentinel2L2a,
    LandsatC2L2,
}

impl Collection {
    pub fn id(self) -> &'static str {
        match self {
            Collection::Sentinel2L2a => "sentinel-2-l2a",
            Collection::LandsatC2L2 => "landsat-c2-l2",
        }
    }
    pub fn source(self) -> &'static str {
        match self {
            Collection::Sentinel2L2a => "Sentinel-2 L2A",
            Collection::LandsatC2L2 => "Landsat C2 L2",
        }
    }
    pub fn resolution_m(self) -> f64 {
        match self {
            Collection::Sentinel2L2a => 10.0,
            Collection::LandsatC2L2 => 30.0,
        }
    }
    /// Credit line required by the data licence.
    pub fn credit(self, year: i32) -> String {
        match self {
            Collection::Sentinel2L2a => format!("Contains modified Copernicus Sentinel data {year}, {PC_CREDIT}"),
            Collection::LandsatC2L2 => format!("Landsat courtesy of the U.S. Geological Survey, {PC_CREDIT}"),
        }
    }
    pub fn license(self) -> &'static str {
        match self {
            Collection::Sentinel2L2a => "Copernicus Sentinel data terms (free, full and open)",
            Collection::LandsatC2L2 => "USGS Landsat data: public domain",
        }
    }
    /// Parses "s2", "sentinel-2", "landsat", ...; "all" or empty gives both.
    pub fn parse_list(s: Option<&str>) -> Vec<Collection> {
        match s.map(|s| s.to_ascii_lowercase()) {
            Some(s) if s.starts_with("s2") || s.starts_with("sentinel") => vec![Collection::Sentinel2L2a],
            Some(s) if s.starts_with("l") => vec![Collection::LandsatC2L2],
            _ => vec![Collection::Sentinel2L2a, Collection::LandsatC2L2],
        }
    }
}

/// One satellite scene covering the field.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub collection: Collection,
    pub source: String,
    /// Platform, e.g. "Sentinel-2B", "landsat-9".
    pub platform: String,
    pub datetime: String,
    pub date: NaiveDate,
    /// Whole-scene cloud cover from the STAC metadata (`eo:cloud_cover`), percent.
    pub scene_cloud_pct: f64,
    pub resolution_m: f64,
    /// Sentinel-2 processing baseline (>= 04.00 adds a -1000 DN offset to reflectances).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_baseline: Option<String>,
    pub credit: String,
}

impl Scene {
    /// True when Sentinel-2 digital numbers carry the +1000 offset (processing baseline 04.00 and later).
    pub fn s2_has_offset(&self) -> bool {
        match &self.processing_baseline {
            Some(b) => b.split('.').next().and_then(|m| m.parse::<u32>().ok()).map(|m| m >= 4).unwrap_or(true),
            // Baseline 04.00 started on 2022-01-25.
            None => self.date >= NaiveDate::from_ymd_opt(2022, 1, 25).unwrap(),
        }
    }
}

/// Searches one collection for scenes intersecting the field between `from` and `to` with scene cloud below `max_cloud`.
pub async fn search(
    http: &reqwest::Client,
    field: &Field,
    collection: Collection,
    from: NaiveDate,
    to: NaiveDate,
    max_cloud: f64,
) -> anyhow::Result<Vec<Scene>> {
    let geometry = if field.ring.len() >= 3 {
        pc::polygon_geometry(&pc::field_ring(field))
    } else {
        json!({ "type": "Point", "coordinates": [field.center.lon, field.center.lat] })
    };
    let mut query = json!({ "eo:cloud_cover": { "lt": max_cloud } });
    if collection == Collection::LandsatC2L2 {
        query["platform"] = json!({ "in": ["landsat-8", "landsat-9"] });
    }
    let body = json!({
        "collections": [collection.id()],
        "intersects": geometry,
        "datetime": format!("{from}T00:00:00Z/{to}T23:59:59Z"),
        "query": query,
        "limit": 250,
        "sortby": [{ "field": "datetime", "direction": "desc" }],
        "fields": {
            "include": ["id", "collection", "properties.datetime", "properties.eo:cloud_cover",
                        "properties.platform", "properties.s2:processing_baseline"],
            "exclude": ["assets", "links", "geometry", "bbox"]
        }
    });
    let feats = pc::stac_search(http, body, 8).await?;
    let mut scenes: Vec<Scene> = feats.iter().filter_map(|f| parse_scene(f, collection)).collect();
    scenes.sort_by(|a, b| b.datetime.cmp(&a.datetime));
    Ok(scenes)
}

fn parse_scene(f: &Value, collection: Collection) -> Option<Scene> {
    let p = &f["properties"];
    let datetime = p["datetime"].as_str()?.to_string();
    let date = NaiveDate::parse_from_str(datetime.get(..10)?, "%Y-%m-%d").ok()?;
    use chrono::Datelike;
    Some(Scene {
        id: f["id"].as_str()?.to_string(),
        collection,
        source: collection.source().into(),
        platform: p["platform"].as_str().unwrap_or("").to_string(),
        datetime,
        date,
        scene_cloud_pct: p["eo:cloud_cover"].as_f64().unwrap_or(f64::NAN),
        resolution_m: collection.resolution_m(),
        processing_baseline: p["s2:processing_baseline"].as_str().map(String::from),
        credit: collection.credit(date.year()),
    })
}

/// Both collections merged, newest first.
pub async fn search_all(
    http: &reqwest::Client,
    field: &Field,
    collections: &[Collection],
    from: NaiveDate,
    to: NaiveDate,
    max_cloud: f64,
) -> anyhow::Result<Vec<Scene>> {
    let mut all = Vec::new();
    for c in collections {
        all.extend(search(http, field, *c, from, to, max_cloud).await?);
    }
    all.sort_by(|a, b| b.datetime.cmp(&a.datetime));
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s2_offset_by_baseline() {
        let mut s = Scene {
            id: "x".into(), collection: Collection::Sentinel2L2a, source: String::new(), platform: String::new(),
            datetime: "2021-06-01T00:00:00Z".into(), date: NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
            scene_cloud_pct: 0.0, resolution_m: 10.0, processing_baseline: Some("03.00".into()), credit: String::new(),
        };
        assert!(!s.s2_has_offset());
        s.processing_baseline = Some("05.13".into());
        assert!(s.s2_has_offset());
        s.processing_baseline = None;
        assert!(!s.s2_has_offset());
    }

    #[test]
    fn credits() {
        assert!(Collection::Sentinel2L2a.credit(2026).starts_with("Contains modified Copernicus Sentinel data 2026"));
        assert!(Collection::LandsatC2L2.credit(2026).starts_with("Landsat courtesy of the U.S. Geological Survey"));
    }
}
