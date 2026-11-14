//! MariaDB access for analyses, fields, messages, sections and evidence reports.
use crate::model::*;
use common::{geo::Field, ApiError, ApiResult};
use serde_json::Value;
use sqlx::{MySqlPool, Row};

pub async fn create(db: &MySqlPool, n: &NewAnalysis, visitor: Option<&str>) -> ApiResult<String> {
    if n.field.center.lat.abs() > 90.0 || n.field.center.lon.abs() > 180.0 {
        return Err(ApiError::BadRequest("coordinates out of range".into()));
    }
    if n.field.ring.len() > 500 {
        return Err(ApiError::BadRequest("outline has too many points".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let lang = if n.lang == "es" { "es" } else { "en" };
    let mut tx = db.begin().await?;
    sqlx::query("INSERT INTO analyses (id, location_mode, example_site, lang, visitor) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(&n.location_mode).bind(&n.example_site).bind(lang).bind(visitor)
        .execute(&mut *tx).await?;
    let label = n.label.clone().unwrap_or_else(|| format!("{:.4}, {:.4}", n.field.center.lat, n.field.center.lon));
    sqlx::query("INSERT INTO fields (analysis_id, center_lat, center_lon, outline, area_ha, label) VALUES (?, ?, ?, ST_GeomFromText(?), ?, ?)")
        .bind(&id).bind(n.field.center.lat).bind(n.field.center.lon).bind(n.field.wkt()).bind(n.field.area_ha()).bind(&label)
        .execute(&mut *tx).await?;
    for key in SECTION_KEYS {
        let state = if key == "sheet" { "locked" } else { "empty" };
        sqlx::query("INSERT INTO sections (analysis_id, section, state) VALUES (?, ?, ?)")
            .bind(&id).bind(key).bind(state).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(id)
}

pub async fn field(db: &MySqlPool, id: &str) -> ApiResult<FieldOut> {
    let row = sqlx::query("SELECT center_lat, center_lon, ST_AsText(outline) AS wkt, area_ha, label FROM fields WHERE analysis_id = ? ORDER BY id DESC LIMIT 1")
        .bind(id).fetch_optional(db).await?.ok_or(ApiError::NotFound)?;
    let area: f64 = row.try_get("area_ha")?;
    let wkt: String = row.try_get("wkt")?;
    // A point field is stored as a small square; only expose the ring when the farmer drew one (area recorded > 0).
    let ring = if area > 0.0 { parse_ring(&wkt) } else { vec![] };
    Ok(FieldOut {
        center: common::geo::LatLon { lat: row.try_get("center_lat")?, lon: row.try_get("center_lon")? },
        ring,
        area_ha: (area * 100.0).round() / 100.0,
        label: row.try_get::<Option<String>, _>("label")?.unwrap_or_default(),
    })
}

fn parse_ring(wkt: &str) -> Vec<[f64; 2]> {
    let inner = wkt.trim_start_matches("POLYGON((").trim_end_matches("))");
    let mut pts: Vec<[f64; 2]> = inner.split(',').filter_map(|p| {
        let mut it = p.split_whitespace().map(|x| x.parse::<f64>());
        Some([it.next()?.ok()?, it.next()?.ok()?])
    }).collect();
    if pts.len() > 1 && pts.first() == pts.last() { pts.pop(); }
    pts
}

pub async fn load(db: &MySqlPool, id: &str) -> ApiResult<AnalysisState> { load_raw(db, id).await.map(with_prediagnosis) }

async fn load_raw(db: &MySqlPool, id: &str) -> ApiResult<AnalysisState> {
    let a = sqlx::query("SELECT lang, location_mode, example_site, crop_current_record, crop_history, demo_mode, steps FROM analyses WHERE id = ?")
        .bind(id).fetch_optional(db).await?.ok_or(ApiError::NotFound)?;
    let history: Option<Value> = a.try_get("crop_history")?;
    Ok(AnalysisState {
        id: id.to_string(),
        lang: a.try_get("lang")?,
        location_mode: a.try_get("location_mode")?,
        example_site: a.try_get("example_site")?,
        field: field(db, id).await?,
        crop_current: a.try_get("crop_current_record")?,
        crop_history: history.and_then(|v| v.as_array().cloned()).unwrap_or_default(),
        messages: messages(db, id).await?,
        prediagnosis: None,
        steps: vec![],
        next_step: None,
        sections: sections(db, id).await?,
        evidence: latest_evidence(db, id).await?,
        demo_mode: a.try_get::<i8, _>("demo_mode")? != 0,
    }).map(|mut st: AnalysisState| {
        let stored: Option<Value> = a.try_get("steps").ok().flatten();
        let (steps, next) = crate::steps::from_stored(stored.as_ref());
        st.steps = steps;
        st.next_step = next;
        st
    })
}

/// Fills `prediagnosis` from the rotation section content.
pub fn with_prediagnosis(mut st: AnalysisState) -> AnalysisState {
    st.prediagnosis = st.sections.iter().find(|s| s.key == "rotation").and_then(|s| s.content.as_ref()).and_then(|c| c.get("prediagnosis").cloned()).filter(|v| !v.is_null());
    st
}

pub async fn messages(db: &MySqlPool, id: &str) -> ApiResult<Vec<ChatMessage>> {
    let rows = sqlx::query("SELECT id, role, body, meta, DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%sZ') AS ts FROM messages WHERE analysis_id = ? ORDER BY id")
        .bind(id).fetch_all(db).await?;
    rows.into_iter().map(|r| {
        let meta: Option<Value> = r.try_get("meta")?;
        let photos = meta.as_ref().and_then(|m| m.get("photos")).and_then(|p| serde_json::from_value(p.clone()).ok()).unwrap_or_default();
        Ok(ChatMessage { id: r.try_get("id")?, role: r.try_get("role")?, body: r.try_get("body")?, photos, created_at: r.try_get("ts")? })
    }).collect()
}

pub async fn add_message(db: &MySqlPool, id: &str, role: &str, body: &str, photos: &[PhotoRef]) -> ApiResult<i64> {
    let meta = if photos.is_empty() { None } else { Some(serde_json::json!({ "photos": photos })) };
    let r = sqlx::query("INSERT INTO messages (analysis_id, role, body, meta) VALUES (?, ?, ?, ?)")
        .bind(id).bind(role).bind(body).bind(meta).execute(db).await?;
    Ok(r.last_insert_id() as i64)
}

pub async fn sections(db: &MySqlPool, id: &str) -> ApiResult<Vec<Section>> {
    let rows = sqlx::query("SELECT section, state, content FROM sections WHERE analysis_id = ?").bind(id).fetch_all(db).await?;
    let mut out: Vec<Section> = rows.into_iter().map(|r| -> ApiResult<Section> {
        let content: Option<Value> = r.try_get("content")?;
        let note = content.as_ref().and_then(|c| c.get("note")).and_then(|n| serde_json::from_value(n.clone()).ok());
        let content = content.and_then(|c| c.get("content").cloned()).filter(|c| !c.is_null());
        Ok(Section { key: r.try_get("section")?, state: r.try_get("state")?, note, content })
    }).collect::<ApiResult<_>>()?;
    out.sort_by_key(|s| SECTION_KEYS.iter().position(|k| *k == s.key).unwrap_or(99));
    Ok(out)
}

pub async fn set_section(db: &MySqlPool, id: &str, key: &str, state: &str, note: Option<&Bi>, content: Option<&Value>) -> ApiResult<()> {
    if !SECTION_KEYS.contains(&key) { return Err(ApiError::BadRequest(format!("unknown section {key}"))); }
    let blob = serde_json::json!({ "note": note, "content": content });
    sqlx::query("REPLACE INTO sections (analysis_id, section, state, content) VALUES (?, ?, ?, ?)")
        .bind(id).bind(key).bind(state).bind(blob).execute(db).await?;
    Ok(())
}

pub async fn set_crops(db: &MySqlPool, id: &str, current: Option<&Value>, history: Option<&[Value]>) -> ApiResult<()> {
    if let Some(c) = current {
        let name = c.pointer("/names/en").and_then(Value::as_str).unwrap_or_default();
        sqlx::query("UPDATE analyses SET crop_current_record = ?, crop_current = ? WHERE id = ?").bind(c).bind(name).bind(id).execute(db).await?;
    }
    if let Some(h) = history {
        sqlx::query("UPDATE analyses SET crop_history = ? WHERE id = ?").bind(Value::from(h.to_vec())).bind(id).execute(db).await?;
    }
    Ok(())
}

pub async fn set_demo_mode(db: &MySqlPool, id: &str, on: bool) -> ApiResult<()> {
    sqlx::query("UPDATE analyses SET demo_mode = ? WHERE id = ?").bind(on as i8).bind(id).execute(db).await?;
    Ok(())
}

pub async fn set_lang(db: &MySqlPool, id: &str, lang: &str) -> ApiResult<()> {
    let lang = if lang == "es" { "es" } else { "en" };
    sqlx::query("UPDATE analyses SET lang = ? WHERE id = ?").bind(lang).bind(id).execute(db).await?;
    Ok(())
}

pub async fn save_evidence(db: &MySqlPool, id: &str, evidence: &Value) -> ApiResult<()> {
    sqlx::query("INSERT INTO reports (analysis_id, evidence) VALUES (?, ?)").bind(id).bind(evidence).execute(db).await?;
    Ok(())
}

pub async fn latest_evidence(db: &MySqlPool, id: &str) -> ApiResult<Option<Value>> {
    Ok(sqlx::query_scalar("SELECT evidence FROM reports WHERE analysis_id = ? ORDER BY id DESC LIMIT 1").bind(id).fetch_optional(db).await?)
}

pub async fn share_token(db: &MySqlPool, id: &str) -> ApiResult<String> {
    let existing: Option<Option<String>> = sqlx::query_scalar("SELECT share_token FROM analyses WHERE id = ?").bind(id).fetch_optional(db).await?;
    match existing {
        None => Err(ApiError::NotFound),
        Some(Some(t)) => Ok(t),
        Some(None) => {
            let t: String = uuid::Uuid::new_v4().simple().to_string().chars().take(22).collect();
            sqlx::query("UPDATE analyses SET share_token = ? WHERE id = ?").bind(&t).bind(id).execute(db).await?;
            Ok(t)
        }
    }
}

pub async fn by_share_token(db: &MySqlPool, token: &str) -> ApiResult<String> {
    sqlx::query_scalar("SELECT id FROM analyses WHERE share_token = ?").bind(token).fetch_optional(db).await?.ok_or(ApiError::NotFound)
}

/// Field geometry for services: the stored centre and ring.
pub async fn field_geom(db: &MySqlPool, id: &str) -> ApiResult<Field> { Ok(field(db, id).await?.geom()) }

/// Retention: delete analyses older than `days`, except example-site ones created by the seeding step (visitor 'seed').
pub async fn purge_older_than(db: &MySqlPool, days: i64) -> ApiResult<u64> {
    let r = sqlx::query("DELETE FROM analyses WHERE created_at < NOW() - INTERVAL ? DAY AND (visitor IS NULL OR visitor <> 'seed')")
        .bind(days).execute(db).await?;
    Ok(r.rows_affected())
}

/// Marks a guided topic done with a one-line summary; its result section becomes "ready" (the sheet when all are done).
pub async fn complete_step(db: &MySqlPool, id: &str, key: &str, summary: &Bi) -> ApiResult<Option<String>> {
    if !crate::steps::ORDER[..5].contains(&key) { return Err(ApiError::BadRequest(format!("unknown topic {key}"))); }
    let stored: Option<Value> = sqlx::query_scalar("SELECT steps FROM analyses WHERE id = ?").bind(id).fetch_optional(db).await?.ok_or(ApiError::NotFound)?;
    let v = crate::steps::mark_done(stored, key, summary);
    sqlx::query("UPDATE analyses SET steps = ? WHERE id = ?").bind(&v).bind(id).execute(db).await?;
    sqlx::query("UPDATE sections SET state = 'ready' WHERE analysis_id = ? AND section = ? AND state <> 'gaps'").bind(id).bind(key).execute(db).await?;
    let (_, next) = crate::steps::from_stored(Some(&v));
    if next.is_none() {
        sqlx::query("UPDATE sections SET state = 'ready' WHERE analysis_id = ? AND section = 'sheet'").bind(id).execute(db).await?;
    }
    Ok(next)
}

/// The consultant's first message, written by the backend so the farmer sees at once what the app knows and why.
pub async fn add_greeting(db: &MySqlPool, id: &str, lang: &str) -> ApiResult<()> {
    let f = field(db, id).await?;
    let (lat, lon) = (f.center.lat, f.center.lon);
    let coord = format!("{:.2}°{} {:.2}°{}", lat.abs(), if lat < 0.0 { "S" } else { "N" }, lon.abs(), if lon < 0.0 { "W" } else { "E" });
    let area = if f.area_ha > 0.0 { format!(", {:.2} ha", f.area_ha) } else { String::new() };
    let place = if lang == "es" && f.label == "Huaral valley, Peru" { "Valle de Huaral, Perú".to_string() } else { f.label.clone() };
    let body = if lang == "es" {
        format!("¡Hola! Ubiqué tu campo en **{coord}** ({}{area}). Lo que te mostraré son datos de la NASA y de satélites sobre **la parcela que marcaste en el mapa**, no sobre ti.

Vamos a ver cinco temas, uno por uno: **tu campo**, **suelo**, **manejo de plagas**, **nutrición** y **rotación**. Al terminar tendrás tu ficha técnica para tu agrónomo.

Para empezar: ¿qué cultivas ahora y qué hubo antes en esta parcela?", place)
    } else {
        format!("Hello! I located your field at **{coord}** ({}{area}). What I'll show you is NASA and satellite data about **the plot you marked on the map**, not about you.

We'll go through five topics, one at a time: **your field**, **soil**, **pest care**, **nutrition** and **rotation**. At the end you'll have a technical sheet for your agronomist.

To start: what are you growing now, and what grew on this plot before?", place)
    };
    add_message(db, id, "assistant", &body, &[]).await?;
    Ok(())
}
