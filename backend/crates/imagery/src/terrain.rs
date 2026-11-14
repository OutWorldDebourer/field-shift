//! NASADEM elevation and slope for a field: native 1-arc-second grid from the data API (.npy), slope by Horn's method.
use crate::{pc, stac::BADGE};
use anyhow::{anyhow, bail, Context};
use common::geo::Field;
use serde::{Deserialize, Serialize};
use serde_json::json;

const ARCSEC: f64 = 1.0 / 3600.0;
const MAX_CELLS: usize = 400;
pub const CREDIT: &str = "NASADEM (NASA JPL, public domain), processed by Microsoft Planetary Computer";
pub const METHOD: &str = "NASADEM 1-arc-second (~30 m) elevation read at native resolution from the Planetary Computer \
data API; slope per cell by Horn's 3x3 method with cell sizes in metres at the field latitude; mean and max over \
the cells whose centres fall inside the field (nearest cell for very small fields).";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Terrain {
    pub elevation_mean_m: f64,
    pub elevation_min_m: f64,
    pub elevation_max_m: f64,
    pub slope_mean_deg: f64,
    pub slope_max_deg: f64,
    pub cells: usize,
    pub source: String,
    pub scene_id: String,
    /// NASADEM is a reprocessing of the SRTM flight (February 2000).
    pub date: String,
    pub resolution_m: f64,
    pub badge: String,
    pub credit: String,
    pub license: String,
    pub method: String,
}

/// A parsed `.npy` array as f64 values with its shape.
#[derive(Debug)]
pub struct NpyArray {
    pub shape: Vec<usize>,
    pub data: Vec<f64>,
}

/// Minimal NumPy .npy v1/v2 reader for little-endian numeric arrays in C order.
pub fn parse_npy(b: &[u8]) -> anyhow::Result<NpyArray> {
    if b.len() < 10 || &b[..6] != b"\x93NUMPY" {
        bail!("not an npy file");
    }
    let (hlen, start) = match b[6] {
        1 => (u16::from_le_bytes([b[8], b[9]]) as usize, 10),
        _ => (u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize, 12),
    };
    let header = std::str::from_utf8(b.get(start..start + hlen).ok_or_else(|| anyhow!("npy header truncated"))?)?;
    let descr = header.split("'descr':").nth(1).and_then(|s| s.split('\'').nth(1)).ok_or_else(|| anyhow!("npy descr"))?;
    if header.contains("'fortran_order': True") {
        bail!("fortran-ordered npy not supported");
    }
    let shape_txt = header.split("'shape':").nth(1).and_then(|s| s.split('(').nth(1)).and_then(|s| s.split(')').next())
        .ok_or_else(|| anyhow!("npy shape"))?;
    let shape: Vec<usize> = shape_txt.split(',').map(str::trim).filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>()).collect::<Result<_, _>>()?;
    let n: usize = shape.iter().product();
    let raw = &b[start + hlen..];
    let (size, conv): (usize, fn(&[u8]) -> f64) = match descr {
        "|u1" | "<u1" => (1, |c| c[0] as f64),
        "|i1" | "<i1" => (1, |c| c[0] as i8 as f64),
        "<i2" => (2, |c| i16::from_le_bytes([c[0], c[1]]) as f64),
        "<u2" => (2, |c| u16::from_le_bytes([c[0], c[1]]) as f64),
        "<i4" => (4, |c| i32::from_le_bytes(c.try_into().unwrap()) as f64),
        "<u4" => (4, |c| u32::from_le_bytes(c.try_into().unwrap()) as f64),
        "<f4" => (4, |c| f32::from_le_bytes(c.try_into().unwrap()) as f64),
        "<f8" => (8, |c| f64::from_le_bytes(c.try_into().unwrap())),
        other => bail!("npy dtype {other} not supported"),
    };
    if raw.len() < n * size {
        bail!("npy data truncated");
    }
    Ok(NpyArray { shape, data: raw[..n * size].chunks_exact(size).map(conv).collect() })
}

/// Horn slope in degrees for each interior cell of a row-major `ny` x `nx` grid (None on the border or with gaps).
pub fn slope_grid(z: &[f64], valid: &[bool], nx: usize, ny: usize, dx_m: f64, dy_m: f64) -> Vec<Option<f64>> {
    let at = |r: usize, c: usize| z[r * nx + c];
    let mut out = vec![None; nx * ny];
    for r in 1..ny.saturating_sub(1) {
        for c in 1..nx.saturating_sub(1) {
            if (r - 1..=r + 1).any(|rr| (c - 1..=c + 1).any(|cc| !valid[rr * nx + cc])) {
                continue;
            }
            let dzdx = ((at(r - 1, c + 1) + 2.0 * at(r, c + 1) + at(r + 1, c + 1))
                - (at(r - 1, c - 1) + 2.0 * at(r, c - 1) + at(r + 1, c - 1))) / (8.0 * dx_m);
            let dzdy = ((at(r + 1, c - 1) + 2.0 * at(r + 1, c) + at(r + 1, c + 1))
                - (at(r - 1, c - 1) + 2.0 * at(r - 1, c) + at(r - 1, c + 1))) / (8.0 * dy_m);
            out[r * nx + c] = Some((dzdx.hypot(dzdy)).atan().to_degrees());
        }
    }
    out
}

/// Elevation and slope over the field from NASADEM (no database cache).
pub async fn terrain_uncached(http: &reqwest::Client, field: &Field) -> anyhow::Result<Terrain> {
    let c = field.center;
    let items = pc::stac_search(http, json!({
        "collections": ["nasadem"],
        "intersects": { "type": "Point", "coordinates": [c.lon, c.lat] },
        "limit": 1
    }), 1).await?;
    let item = items.first().and_then(|f| f["id"].as_str()).ok_or_else(|| anyhow!("no NASADEM tile at this point"))?.to_string();

    let poly = pc::field_as_polygon(field);
    let [w, s, e, n] = poly.bbox(0.0);
    // Align to NASADEM pixel edges (centres on whole arc-seconds) and pad 2 cells for the 3x3 kernel.
    let edge = |x: f64, up: bool| {
        let k = x / ARCSEC + 0.5;
        let k = if up { k.ceil() } else { k.floor() };
        k * ARCSEC - 0.5 * ARCSEC
    };
    let (minx, maxx) = (edge(w, false) - 2.0 * ARCSEC, edge(e, true) + 2.0 * ARCSEC);
    let (miny, maxy) = (edge(s, false) - 2.0 * ARCSEC, edge(n, true) + 2.0 * ARCSEC);
    let nx = ((maxx - minx) / ARCSEC).round() as usize;
    let ny = ((maxy - miny) / ARCSEC).round() as usize;
    if nx > MAX_CELLS || ny > MAX_CELLS {
        bail!("field too large for the terrain grid ({nx}x{ny} cells)");
    }
    let url = format!("{}/item/bbox/{minx:.9},{miny:.9},{maxx:.9},{maxy:.9}/{nx}x{ny}.npy", pc::DATA);
    let bytes = pc::get_bytes(http, &url, &[("collection", "nasadem"), ("item", &item), ("assets", "elevation")]).await?;
    let arr = parse_npy(&bytes).context("NASADEM grid")?;
    let (bands, rows, cols) = match arr.shape.as_slice() {
        [b, r, c] => (*b, *r, *c),
        [r, c] => (1, *r, *c),
        _ => bail!("unexpected NASADEM grid shape {:?}", arr.shape),
    };
    if rows != ny || cols != nx {
        tracing::warn!("NASADEM grid {cols}x{rows}, asked {nx}x{ny}");
    }
    let z = &arr.data[..rows * cols];
    // The data API appends a mask band (0 = no data).
    let valid: Vec<bool> = if bands >= 2 {
        arr.data[rows * cols..2 * rows * cols].iter().map(|m| *m > 0.0).collect()
    } else {
        z.iter().map(|v| *v > -1000.0).collect()
    };
    let lat = c.lat.to_radians();
    let dy_m = (maxy - miny) / rows as f64 * 111_320.0;
    let dx_m = (maxx - minx) / cols as f64 * 111_320.0 * lat.cos();
    let slopes = slope_grid(z, &valid, cols, rows, dx_m, dy_m);

    let centre = |r: usize, col: usize| (minx + (col as f64 + 0.5) * (maxx - minx) / cols as f64, maxy - (r as f64 + 0.5) * (maxy - miny) / rows as f64);
    let mut picked: Vec<usize> = (0..rows * cols)
        .filter(|&i| slopes[i].is_some() && { let (x, y) = centre(i / cols, i % cols); poly.contains(x, y) })
        .collect();
    if picked.is_empty() {
        let nearest = (0..rows * cols).filter(|&i| slopes[i].is_some()).min_by(|&a, &b| {
            let d = |i: usize| { let (x, y) = centre(i / cols, i % cols); (x - c.lon).powi(2) + (y - c.lat).powi(2) };
            d(a).total_cmp(&d(b))
        });
        picked.extend(nearest);
    }
    if picked.is_empty() {
        bail!("no valid NASADEM cells over the field");
    }
    let sl: Vec<f64> = picked.iter().filter_map(|&i| slopes[i]).collect();
    let el: Vec<f64> = picked.iter().map(|&i| z[i]).collect();
    let r1 = |x: f64| (x * 10.0).round() / 10.0;
    Ok(Terrain {
        elevation_mean_m: r1(el.iter().sum::<f64>() / el.len() as f64),
        elevation_min_m: el.iter().cloned().fold(f64::MAX, f64::min),
        elevation_max_m: el.iter().cloned().fold(f64::MIN, f64::max),
        slope_mean_deg: r1(sl.iter().sum::<f64>() / sl.len() as f64),
        slope_max_deg: r1(sl.iter().cloned().fold(f64::MIN, f64::max)),
        cells: picked.len(),
        source: "NASADEM".into(),
        scene_id: item,
        date: "2000-02-20".into(),
        resolution_m: 30.0,
        badge: BADGE.into(),
        credit: CREDIT.into(),
        license: "NASA data: public domain".into(),
        method: METHOD.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horn_slope_of_a_plane() {
        // z rises 3 m per 30 m cell eastwards: gradient 0.1 -> 5.71 degrees.
        let (nx, ny) = (5, 4);
        let z: Vec<f64> = (0..nx * ny).map(|i| 3.0 * (i % nx) as f64).collect();
        let s = slope_grid(&z, &vec![true; nx * ny], nx, ny, 30.0, 30.0);
        assert!(s[0].is_none());
        let v = s[nx + 1].unwrap();
        assert!((v - 0.1f64.atan().to_degrees()).abs() < 1e-9, "{v}");
        let flat = slope_grid(&vec![100.0; 9], &vec![true; 9], 3, 3, 30.0, 30.0);
        assert_eq!(flat[4], Some(0.0));
    }

    #[test]
    fn npy_int16_with_mask() {
        let header = "{'descr': '<i2', 'fortran_order': False, 'shape': (2, 1, 2), }";
        let mut h = header.to_string();
        while (10 + h.len() + 1) % 64 != 0 { h.push(' '); }
        h.push('\n');
        let mut b = b"\x93NUMPY\x01\x00".to_vec();
        b.extend_from_slice(&(h.len() as u16).to_le_bytes());
        b.extend_from_slice(h.as_bytes());
        for v in [186i16, -5, 255, 0] { b.extend_from_slice(&v.to_le_bytes()); }
        let a = parse_npy(&b).unwrap();
        assert_eq!(a.shape, vec![2, 1, 2]);
        assert_eq!(a.data, vec![186.0, -5.0, 255.0, 0.0]);
    }
}
