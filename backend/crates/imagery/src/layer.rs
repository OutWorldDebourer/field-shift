//! Data layers for the dashboard map, drawn from the measurements themselves:
//! - `ndvi`: crop vigor per 10 m pixel from the clearest recent Sentinel-2 L2A scene, coloured red (bare soil) to
//!   green (dense crop) by the Planetary Computer data API;
//! - `slope`: slope classes per 30 m NASADEM cell, computed here with Horn's 3x3 method on the native grid (a slope
//!   computed on a resampled image is noise on flat land), classes grouped from FAO Guidelines for soil description.
//! Each layer is a PNG plus metadata (the box to place it, date, source, licence), cached on disk for 24 h.
use crate::{ndvi, pc, snapshot, terrain};
use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerMeta {
    pub kind: String,
    /// [west, south, east, north] of the image, degrees.
    pub bbox: [f64; 4],
    pub date: Option<String>,
    pub source: String,
    pub resolution: String,
    pub badge: String,
    pub license: String,
    pub credit: String,
    pub method: String,
    pub image_url: String,
}

/// Slope classes (percent) and colours, flat to very steep. 0–3 % merges FAO's flat to gently sloping classes,
/// which the 1 m vertical steps of NASADEM cannot separate.
pub const SLOPE_CLASSES: [(f64, [u8; 4]); 5] = [
    (3.0, [126, 200, 140, 70]),
    (8.0, [236, 216, 110, 150]),
    (15.0, [244, 166, 72, 175]),
    (30.0, [226, 92, 58, 195]),
    (f64::INFINITY, [160, 36, 64, 210]),
];

fn paths(root: &Path, kind: &str, lat: f64, lon: f64, size_m: f64) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = snapshot::cache_dir(root);
    let stem = format!("layer_{kind}_{lat:.5}_{lon:.5}_{size_m}");
    (dir.join(format!("{stem}.png")), dir.join(format!("{stem}.json")))
}

pub async fn layer(http: &reqwest::Client, root: &Path, kind: &str, lat: f64, lon: f64, size_m: f64) -> anyhow::Result<(Vec<u8>, LayerMeta)> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) { bail!("lat/lon out of range"); }
    let size_m = size_m.clamp(300.0, 10_000.0).round();
    let (png_path, json_path) = paths(root, kind, lat, lon, size_m);
    if snapshot::fresh(&json_path) && png_path.exists() {
        if let (Ok(png), Ok(m)) = (std::fs::read(&png_path), std::fs::read(&json_path)) {
            if let Ok(meta) = serde_json::from_slice::<LayerMeta>(&m) { return Ok((png, meta)); }
        }
    }
    let (png, meta) = match kind {
        "ndvi" => ndvi_layer(http, lat, lon, size_m).await?,
        "slope" => slope_layer(http, lat, lon, size_m).await?,
        _ => bail!("unknown layer {kind}"),
    };
    let _ = std::fs::create_dir_all(snapshot::cache_dir(root))
        .and_then(|_| std::fs::write(&png_path, &png))
        .and_then(|_| std::fs::write(&json_path, serde_json::to_vec(&meta).unwrap_or_default()));
    Ok((png, meta))
}

fn url(kind: &str, lat: f64, lon: f64, size_m: f64) -> String {
    format!("/api/imagery/layer.png?kind={kind}&lat={lat}&lon={lon}&size_m={size_m}")
}

async fn ndvi_layer(http: &reqwest::Client, lat: f64, lon: f64, size_m: f64) -> anyhow::Result<(Vec<u8>, LayerMeta)> {
    let bbox = pc::bbox_around(lat, lon, size_m);
    let (scene, _) = snapshot::pick_scene(http, lat, lon, bbox).await?;
    let full = ndvi::expression(&scene);
    let expr = full.split(';').next().unwrap_or(&full).to_string();
    let [w, s, e, n] = bbox;
    let px = 512;
    let u = format!("{}/item/bbox/{w:.6},{s:.6},{e:.6},{n:.6}/{px}x{px}.png", pc::DATA);
    let png = pc::get_bytes(http, &u, &[
        ("collection", scene.collection.id()), ("item", &scene.id), ("expression", &expr),
        ("asset_as_band", "true"), ("rescale", "0,0.8"), ("colormap_name", "rdylgn"),
    ]).await?;
    if !png.starts_with(b"\x89PNG") { bail!("data API did not return a PNG"); }
    Ok((png, LayerMeta {
        kind: "ndvi".into(), bbox, date: Some(scene.date.to_string()), source: scene.source.clone(),
        resolution: "10 m".into(), badge: "field-level".into(), license: scene.collection.license().into(),
        credit: scene.credit.clone(),
        method: format!("NDVI = (NIR − red) / (NIR + red) per pixel, {expr}, colour ramp RdYlGn from 0 (bare soil) to 0.8 (dense crop); clearest recent scene over the box."),
        image_url: url("ndvi", lat, lon, size_m),
    }))
}

async fn slope_layer(http: &reqwest::Client, lat: f64, lon: f64, size_m: f64) -> anyhow::Result<(Vec<u8>, LayerMeta)> {
    const ARCSEC: f64 = 1.0 / 3600.0;
    let items = pc::stac_search(http, json!({
        "collections": ["nasadem"],
        "intersects": { "type": "Point", "coordinates": [lon, lat] },
        "limit": 1
    }), 1).await?;
    let item = items.first().and_then(|f| f["id"].as_str()).ok_or_else(|| anyhow!("no NASADEM tile at this point"))?.to_string();
    let [w, s, e, n] = pc::bbox_around(lat, lon, size_m);
    // Whole NASADEM cells (centres on whole arc-seconds), so each output pixel is one 30 m cell.
    let edge = |x: f64, up: bool| { let k = x / ARCSEC + 0.5; (if up { k.ceil() } else { k.floor() }) * ARCSEC - 0.5 * ARCSEC };
    let (minx, maxx, miny, maxy) = (edge(w, false), edge(e, true), edge(s, false), edge(n, true));
    let nx = ((maxx - minx) / ARCSEC).round() as usize;
    let ny = ((maxy - miny) / ARCSEC).round() as usize;
    if nx > 400 || ny > 400 || nx < 3 || ny < 3 { bail!("slope grid {nx}x{ny} out of range"); }
    let u = format!("{}/item/bbox/{minx:.9},{miny:.9},{maxx:.9},{maxy:.9}/{nx}x{ny}.npy", pc::DATA);
    let bytes = pc::get_bytes(http, &u, &[("collection", "nasadem"), ("item", &item), ("assets", "elevation")]).await?;
    let arr = terrain::parse_npy(&bytes).context("NASADEM grid")?;
    let (bands, rows, cols) = match arr.shape.as_slice() { [b, r, c] => (*b, *r, *c), [r, c] => (1, *r, *c), _ => bail!("unexpected grid shape") };
    let z = &arr.data[..rows * cols];
    let valid: Vec<bool> = if bands >= 2 { arr.data[rows * cols..2 * rows * cols].iter().map(|m| *m > 0.0).collect() } else { vec![true; rows * cols] };
    let dx = ARCSEC * 111_320.0 * lat.to_radians().cos();
    let dy = ARCSEC * 110_574.0;
    let deg = terrain::slope_grid(z, &valid, cols, rows, dx, dy);
    let mut img = image::RgbaImage::new(cols as u32, rows as u32);
    for (i, d) in deg.iter().enumerate() {
        let px = match d {
            Some(d) => { let pct = d.to_radians().tan() * 100.0; SLOPE_CLASSES.iter().find(|(max, _)| pct < *max).map(|c| c.1).unwrap_or([0, 0, 0, 0]) }
            None => [0, 0, 0, 0],
        };
        img.put_pixel((i % cols) as u32, (i / cols) as u32, image::Rgba(px));
    }
    let mut png = vec![];
    image::DynamicImage::ImageRgba8(img).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)?;
    Ok((png, LayerMeta {
        kind: "slope".into(), bbox: [minx, miny, maxx, maxy], date: None, source: "NASADEM (NASA JPL)".into(),
        resolution: "30 m".into(), badge: "field-level".into(), license: "NASA open data, no restrictions".into(),
        credit: "NASADEM_HGT v001, NASA JPL, via Microsoft Planetary Computer".into(),
        method: "Slope per 30 m cell by Horn's 3x3 method on the native NASADEM grid; classes 0–3, 3–8, 8–15, 15–30, >30 % grouped from FAO Guidelines for soil description (2006).".into(),
        image_url: url("slope", lat, lon, size_m),
    }))
}
