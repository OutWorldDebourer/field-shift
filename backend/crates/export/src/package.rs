//! The agronomist's package: sheet.html (relative links), data.csv, the farmer's photos at full resolution,
//! the satellite picture of the field and a README, in one ZIP.
use crate::{csv, sheet};
use analyses::{photos, store};
use common::{ApiResult, AppState};
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, ZipWriter};

pub async fn build(state: &AppState, id: &str, lang: &str) -> ApiResult<Vec<u8>> {
    let st = store::load(&state.db, id).await?;
    let ev = st.evidence.clone().unwrap_or(serde_json::Value::Null);
    let html = sheet::render_mode(&st, lang, "data.csv", sheet::Mode::Zip);
    let mut files: Vec<(String, Vec<u8>)> = vec![
        ("sheet.html".into(), html.into_bytes()),
        ("data.csv".into(), csv::render(&ev).into_bytes()),
    ];
    for m in &st.messages {
        for p in &m.photos {
            if let Some(pid) = p.url.rsplit('/').next().and_then(|x| x.parse::<i64>().ok()) {
                if let Ok((_, bytes)) = photos::read(state, id, pid).await { files.push((sheet::zip_photo_name(&p.url), bytes)); }
            }
        }
    }
    if let Some(url) = ev.pointer("/imagery/snapshot_url").and_then(|v| v.as_str()) {
        // Same parameters as the sheet's picture, served from the imagery cache.
        let q: std::collections::HashMap<String, String> = url.split('?').nth(1).unwrap_or("").split('&')
            .filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let f = |k: &str| q.get(k).and_then(|v| v.parse::<f64>().ok());
        if let (Some(lat), Some(lon)) = (f("lat"), f("lon")) {
            if let Ok((png, _)) = imagery::snapshot(state, lat, lon, f("size_m").unwrap_or(1000.0), f("px").unwrap_or(1024.0) as u32).await {
                files.push(("satellite.png".into(), png));
            }
        }
    }
    let readme = if lang == "es" {
        "Field Shift — paquete para el agrónomo\n\nsheet.html: ficha técnica (ábrala en un navegador; imprímala o guárdela como PDF).\ndata.csv: todas las lecturas y series con fuente, fecha, resolución y licencia.\nphotos/: fotos del agricultor en resolución completa.\nsatellite.png: imagen Sentinel-2 del campo (Contains modified Copernicus Sentinel data).\n\nPrediagnóstico, no un diagnóstico: la decisión final es del agrónomo.\n"
    } else {
        "Field Shift — package for the agronomist\n\nsheet.html: technical sheet (open it in a browser; print or save as PDF).\ndata.csv: every reading and series with source, date, resolution and licence.\nphotos/: the farmer's photos at full resolution.\nsatellite.png: Sentinel-2 picture of the field (Contains modified Copernicus Sentinel data).\n\nPre-diagnosis, not a diagnosis: the agronomist makes the final decision.\n"
    };
    files.push(("README.txt".into(), readme.as_bytes().to_vec()));

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, bytes) in files {
        zip.start_file(name, opts).map_err(|e| anyhow::anyhow!(e))?;
        zip.write_all(&bytes).map_err(|e| anyhow::anyhow!(e))?;
    }
    Ok(zip.finish().map_err(|e| anyhow::anyhow!(e))?.into_inner())
}
