//! True-colour PNG of the area around a field from the latest clear Sentinel-2 scene, cached on disk under `backend/cache/imagery/`.
use crate::{pc, stac::{self, Collection, Scene, BADGE}};
use anyhow::{anyhow, bail};
use chrono::{Duration, Utc};
use common::geo::Field;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CACHE_HOURS: u64 = 24;
const LOOKBACK_DAYS: i64 = 240;
const MAX_CANDIDATES: usize = 12;
/// A scene counts as clear when at most this share of the picture is cloud or shadow (SCL 3, 8, 9, 10).
const CLEAR_PCT: f64 = 2.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub scene_id: String,
    pub date: chrono::NaiveDate,
    pub source: String,
    pub platform: String,
    pub collection: Collection,
    /// [west, south, east, north] in degrees.
    pub bbox: [f64; 4],
    pub size_m: f64,
    pub px: u32,
    /// Cloud or shadow over the picture (%), from the scene classification layer.
    pub cloud_pct: f64,
    pub resolution_m: f64,
    pub badge: String,
    pub credit: String,
    pub license: String,
    pub method: String,
    pub image_url: String,
}

/// Folder for cached pictures: `<root>/backend/cache/imagery`, created on first use.
pub fn cache_dir(root: &Path) -> PathBuf {
    root.join("backend").join("cache").join("imagery")
}

pub(crate) fn fresh(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|age| age.as_secs() < CACHE_HOURS * 3600)
        .unwrap_or(false)
}

async fn cloud_over(http: &reqwest::Client, scene: &Scene, feature: &serde_json::Value) -> anyhow::Result<(f64, f64)> {
    let s = pc::statistics(
        http,
        scene.collection.id(),
        &scene.id,
        "where((SCL==3)|(SCL==8)|(SCL==9)|(SCL==10),1.0,0.0);where(SCL==0,1.0,0.0)",
        feature,
        false,
    )
    .await?;
    Ok((s[0].mean * 100.0, s[1].mean * 100.0))
}

/// Picks the newest Sentinel-2 scene that is clear over the picture area (or the clearest of the recent ones).
pub(crate) async fn pick_scene(http: &reqwest::Client, lat: f64, lon: f64, bbox: [f64; 4]) -> anyhow::Result<(Scene, f64)> {
    let to = Utc::now().date_naive();
    let from = to - Duration::days(LOOKBACK_DAYS);
    let scenes = stac::search(http, &Field::point(lat, lon), Collection::Sentinel2L2a, from, to, 60.0).await?;
    let [w, s, e, n] = bbox;
    let feature = pc::feature(&[[w, s], [e, s], [e, n], [w, n], [w, s]]);
    let mut best: Option<(Scene, f64)> = None;
    for scene in scenes.into_iter().take(MAX_CANDIDATES) {
        let Ok((cloud, nodata)) = cloud_over(http, &scene, &feature).await else { continue };
        if nodata > 0.5 {
            continue;
        }
        if cloud <= CLEAR_PCT {
            return Ok((scene, cloud));
        }
        if best.as_ref().map(|b| cloud < b.1).unwrap_or(true) {
            best = Some((scene, cloud));
        }
    }
    best.ok_or_else(|| anyhow!("no Sentinel-2 scene covers this area in the last {LOOKBACK_DAYS} days"))
}

/// Returns the PNG bytes and their metadata, from the disk cache when younger than 24 h.
pub async fn snapshot(
    http: &reqwest::Client,
    root: &Path,
    lat: f64,
    lon: f64,
    size_m: f64,
    px: u32,
) -> anyhow::Result<(Vec<u8>, SnapshotMeta)> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        bail!("lat/lon out of range");
    }
    let size_m = size_m.clamp(100.0, 10_000.0).round();
    let px = px.clamp(128, 1024);
    let dir = cache_dir(root);
    let stem = format!("s2_{lat:.5}_{lon:.5}_{size_m}_{px}");
    let (png_path, json_path) = (dir.join(format!("{stem}.png")), dir.join(format!("{stem}.json")));
    if fresh(&json_path) && png_path.exists() {
        if let (Ok(png), Ok(meta)) = (std::fs::read(&png_path), std::fs::read(&json_path)) {
            if let Ok(meta) = serde_json::from_slice::<SnapshotMeta>(&meta) {
                return Ok((png, meta));
            }
        }
    }

    let bbox = pc::bbox_around(lat, lon, size_m);
    let (scene, cloud) = pick_scene(http, lat, lon, bbox).await?;
    let [w, s, e, n] = bbox;
    let url = format!("{}/item/bbox/{w:.6},{s:.6},{e:.6},{n:.6}/{px}x{px}.png", pc::DATA);
    let png = pc::get_bytes(
        http,
        &url,
        &[("collection", scene.collection.id()), ("item", &scene.id), ("assets", "visual"), ("asset_bidx", "visual|1,2,3")],
    )
    .await?;
    if !png.starts_with(b"\x89PNG") {
        bail!("data API did not return a PNG");
    }
    let meta = SnapshotMeta {
        scene_id: scene.id.clone(),
        date: scene.date,
        source: scene.source.clone(),
        platform: scene.platform.clone(),
        collection: scene.collection,
        bbox,
        size_m,
        px,
        cloud_pct: (cloud * 10.0).round() / 10.0,
        resolution_m: scene.resolution_m,
        badge: BADGE.into(),
        credit: scene.credit.clone(),
        license: scene.collection.license().into(),
        method: "Sentinel-2 L2A true-colour (TCI 'visual' asset, bands 4-3-2) cropped to the box by the Planetary Computer \
data API; newest scene with at most 2% cloud or shadow in the box (SCL), else the clearest of the last 12."
            .into(),
        image_url: format!("/api/imagery/snapshot.png?lat={lat}&lon={lon}&size_m={size_m}&px={px}"),
    };
    if let Err(e) = std::fs::create_dir_all(&dir)
        .and_then(|_| std::fs::write(&png_path, &png))
        .and_then(|_| std::fs::write(&json_path, serde_json::to_vec_pretty(&meta).unwrap_or_default()))
    {
        tracing::warn!("imagery: could not write snapshot cache: {e}");
    }
    Ok((png, meta))
}
