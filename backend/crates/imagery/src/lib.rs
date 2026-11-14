//! Imagery and terrain for a field through Microsoft Planetary Computer (no account): Sentinel-2 / Landsat NDVI series,
//! bare-soil periods, a true-colour snapshot and NASADEM slope. Mounted at `/api/imagery`.
pub mod layer;
pub mod ndvi;
pub mod pc;
pub mod snapshot;
pub mod stac;
pub mod terrain;

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{Months, NaiveDate, Utc};
use common::{cache, geo::{Field, LatLon}, ApiError, ApiResult, AppState};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};

pub use ndvi::{BareSoilPeriod, CropCycle, DroppedScene, FieldInfo, NdviParams, NdviPoint, NdviSeries};
pub use snapshot::SnapshotMeta;
pub use stac::{Collection, Scene};
pub use terrain::Terrain;

const SOURCE: &str = "planetary-computer";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ndvi", get(ndvi_get).post(ndvi_post))
        .route("/scenes", get(scenes_get).post(scenes_post))
        .route("/terrain", get(terrain_get).post(terrain_post))
        .route("/snapshot.png", get(snapshot_png))
        .route("/layer.png", get(layer_png))
        .route("/layer.json", get(layer_json))
        .route("/snapshot.json", get(snapshot_json))
}

// ---------- public API for other crates ----------

/// NDVI series for the last `months` months with default thresholds (cached 24 h).
pub async fn ndvi_series(state: &AppState, field: &Field, months: u32) -> anyhow::Result<NdviSeries> {
    ndvi_series_with(state, field, &NdviParams { months, ..Default::default() }).await
}

/// NDVI series with explicit parameters (cached 24 h per field and parameter set).
pub async fn ndvi_series_with(state: &AppState, field: &Field, params: &NdviParams) -> anyhow::Result<NdviSeries> {
    let key = format!(
        "imagery:ndvi:v1:{}:{}:{}:{}:{}",
        pc::field_key(field), params.months, params.max_scene_cloud_pct, params.max_field_cloud_pct, params.bare_soil_threshold
    );
    cached(state, &key, 24, || ndvi::ndvi_series_uncached(&state.http, field, params)).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneList {
    pub field: FieldInfo,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub max_scene_cloud_pct: f64,
    pub count: usize,
    pub scenes: Vec<Scene>,
    pub badge: String,
    pub note: String,
}

/// STAC scenes over the field, newest first (cached 24 h). Cloud is the whole-scene STAC value.
pub async fn scenes(state: &AppState, field: &Field, months: u32, max_cloud: f64, collections: &[Collection]) -> anyhow::Result<SceneList> {
    let ids: Vec<&str> = collections.iter().map(|c| c.id()).collect();
    let key = format!("imagery:scenes:v1:{}:{months}:{max_cloud}:{}", pc::field_key(field), ids.join("+"));
    cached(state, &key, 24, || async move {
        let to = Utc::now().date_naive();
        let from = to.checked_sub_months(Months::new(months.clamp(1, 120))).unwrap_or(to);
        let scenes = stac::search_all(&state.http, field, collections, from, to, max_cloud).await?;
        Ok(SceneList {
            field: ndvi::field_info(field),
            from,
            to,
            max_scene_cloud_pct: max_cloud,
            count: scenes.len(),
            scenes,
            badge: stac::BADGE.into(),
            note: "scene_cloud_pct is for the whole scene; /api/imagery/ndvi checks cloud over the field itself.".into(),
        })
    })
    .await
}

/// NASADEM elevation and slope over the field (cached 30 days; the DEM never changes).
pub async fn terrain(state: &AppState, field: &Field) -> anyhow::Result<Terrain> {
    let key = format!("imagery:terrain:v1:{}", pc::field_key(field));
    cached(state, &key, 24 * 30, || terrain::terrain_uncached(&state.http, field)).await
}

/// True-colour PNG around a point and its metadata (disk cache under `backend/cache/imagery`, 24 h).
pub async fn snapshot(state: &AppState, lat: f64, lon: f64, size_m: f64, px: u32) -> anyhow::Result<(Vec<u8>, SnapshotMeta)> {
    snapshot::snapshot(&state.http, &state.config.root, lat, lon, size_m, px).await
}

/// `cache::get_or_fetch`, but still answers (uncached) when the database itself is unreachable.
async fn cached<T, F, Fut>(state: &AppState, key: &str, ttl_hours: i64, fetch: F) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned,
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let ran = AtomicBool::new(false);
    match cache::get_or_fetch(&state.db, SOURCE, key, ttl_hours, || {
        ran.store(true, Ordering::Relaxed);
        fetch()
    })
    .await
    {
        Ok(v) => Ok(v),
        Err(e) if ran.load(Ordering::Relaxed) => Err(e),
        Err(e) => {
            tracing::warn!("imagery: cache unavailable ({e}); fetching without cache");
            fetch().await
        }
    }
}

// ---------- HTTP handlers ----------

/// Query string: `lat`, `lon`, optional `ring=lon,lat;lon,lat;...`, `months`, `max_cloud`, `sensor` (s2|landsat|all), `bare_threshold`.
#[derive(Debug, Deserialize)]
pub struct FieldQuery {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub ring: Option<String>,
    pub months: Option<u32>,
    pub max_cloud: Option<f64>,
    pub sensor: Option<String>,
    pub bare_threshold: Option<f64>,
}

/// JSON body: `{ "center": {"lat":..,"lon":..}, "ring": [[lon,lat],...], "months": 24, ... }`.
#[derive(Debug, Deserialize)]
pub struct FieldBody {
    pub center: Option<LatLon>,
    #[serde(default)]
    pub ring: Vec<[f64; 2]>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub months: Option<u32>,
    pub max_cloud: Option<f64>,
    pub sensor: Option<String>,
    pub bare_threshold: Option<f64>,
}

struct Opts {
    months: Option<u32>,
    max_cloud: Option<f64>,
    sensor: Option<String>,
    bare_threshold: Option<f64>,
}

fn bad(msg: impl Into<String>) -> ApiError {
    ApiError::BadRequest(msg.into())
}

fn upstream(e: anyhow::Error) -> ApiError {
    ApiError::Upstream(format!("{e:#}"))
}

fn parse_ring(s: &str) -> ApiResult<Vec<[f64; 2]>> {
    s.split(';')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (x, y) = p.split_once(',').ok_or_else(|| bad("ring must be lon,lat;lon,lat;..."))?;
            Ok([x.trim().parse().map_err(|_| bad("bad ring longitude"))?, y.trim().parse().map_err(|_| bad("bad ring latitude"))?])
        })
        .collect()
}

/// Builds and validates a field from a centre and/or ring.
pub fn make_field(center: Option<LatLon>, mut ring: Vec<[f64; 2]>) -> ApiResult<Field> {
    if ring.len() > 1 && ring.first() == ring.last() {
        ring.pop();
    }
    if !ring.is_empty() && ring.len() < 3 {
        return Err(bad("ring needs at least 3 vertices"));
    }
    if ring.len() > 1000 {
        return Err(bad("ring has too many vertices (max 1000)"));
    }
    if ring.iter().any(|[x, y]| !(-180.0..=180.0).contains(x) || !(-90.0..=90.0).contains(y)) {
        return Err(bad("ring vertices must be [lon, lat] in degrees"));
    }
    let center = match center {
        Some(c) => c,
        None if !ring.is_empty() => {
            let n = ring.len() as f64;
            LatLon { lat: ring.iter().map(|p| p[1]).sum::<f64>() / n, lon: ring.iter().map(|p| p[0]).sum::<f64>() / n }
        }
        None => return Err(bad("give lat and lon, or a ring")),
    };
    if !(-90.0..=90.0).contains(&center.lat) || !(-180.0..=180.0).contains(&center.lon) {
        return Err(bad("lat/lon out of range"));
    }
    let field = Field { center, ring };
    if field.area_ha() > 5000.0 {
        return Err(bad("field larger than 5000 ha; draw a single field"));
    }
    Ok(field)
}

fn from_query(q: FieldQuery) -> ApiResult<(Field, Opts)> {
    let ring = q.ring.as_deref().map(parse_ring).transpose()?.unwrap_or_default();
    let center = match (q.lat, q.lon) {
        (Some(lat), Some(lon)) => Some(LatLon { lat, lon }),
        _ => None,
    };
    Ok((make_field(center, ring)?, Opts { months: q.months, max_cloud: q.max_cloud, sensor: q.sensor, bare_threshold: q.bare_threshold }))
}

fn from_body(b: FieldBody) -> ApiResult<(Field, Opts)> {
    let center = b.center.or(match (b.lat, b.lon) {
        (Some(lat), Some(lon)) => Some(LatLon { lat, lon }),
        _ => None,
    });
    Ok((make_field(center, b.ring)?, Opts { months: b.months, max_cloud: b.max_cloud, sensor: b.sensor, bare_threshold: b.bare_threshold }))
}

async fn do_ndvi(state: &AppState, field: Field, o: Opts) -> ApiResult<Json<NdviSeries>> {
    let mut p = NdviParams::default();
    if let Some(m) = o.months { p.months = m.clamp(1, 60); }
    if let Some(c) = o.max_cloud { p.max_field_cloud_pct = c.clamp(0.0, 100.0); }
    if let Some(t) = o.bare_threshold { p.bare_soil_threshold = t.clamp(-1.0, 1.0); }
    Ok(Json(ndvi_series_with(state, &field, &p).await.map_err(upstream)?))
}

async fn ndvi_get(State(s): State<AppState>, Query(q): Query<FieldQuery>) -> ApiResult<Json<NdviSeries>> {
    let (f, o) = from_query(q)?;
    do_ndvi(&s, f, o).await
}

async fn ndvi_post(State(s): State<AppState>, Json(b): Json<FieldBody>) -> ApiResult<Json<NdviSeries>> {
    let (f, o) = from_body(b)?;
    do_ndvi(&s, f, o).await
}

async fn do_scenes(state: &AppState, field: Field, o: Opts) -> ApiResult<Json<SceneList>> {
    let months = o.months.unwrap_or(24).clamp(1, 60);
    let max_cloud = o.max_cloud.unwrap_or(70.0).clamp(0.0, 100.0);
    let cols = Collection::parse_list(o.sensor.as_deref());
    Ok(Json(scenes(state, &field, months, max_cloud, &cols).await.map_err(upstream)?))
}

async fn scenes_get(State(s): State<AppState>, Query(q): Query<FieldQuery>) -> ApiResult<Json<SceneList>> {
    let (f, o) = from_query(q)?;
    do_scenes(&s, f, o).await
}

async fn scenes_post(State(s): State<AppState>, Json(b): Json<FieldBody>) -> ApiResult<Json<SceneList>> {
    let (f, o) = from_body(b)?;
    do_scenes(&s, f, o).await
}

async fn terrain_get(State(s): State<AppState>, Query(q): Query<FieldQuery>) -> ApiResult<Json<Terrain>> {
    let (f, _) = from_query(q)?;
    Ok(Json(terrain(&s, &f).await.map_err(upstream)?))
}

async fn terrain_post(State(s): State<AppState>, Json(b): Json<FieldBody>) -> ApiResult<Json<Terrain>> {
    let (f, _) = from_body(b)?;
    Ok(Json(terrain(&s, &f).await.map_err(upstream)?))
}

/// `lat`, `lon`, `size_m` (side of the square box, default 1000, 100..10000), `px` (default 768, 128..1024).
#[derive(Debug, Deserialize)]
pub struct SnapshotQuery {
    pub lat: f64,
    pub lon: f64,
    pub size_m: Option<f64>,
    pub px: Option<u32>,
}

async fn snapshot_png(State(s): State<AppState>, Query(q): Query<SnapshotQuery>) -> ApiResult<impl IntoResponse> {
    let (png, meta) = snapshot(&s, q.lat, q.lon, q.size_m.unwrap_or(1000.0), q.px.unwrap_or(768)).await.map_err(upstream)?;
    let mut h = HeaderMap::new();
    let hv = |v: &str| HeaderValue::from_str(&v.replace(|c: char| !c.is_ascii() || c.is_ascii_control(), " ")).unwrap_or(HeaderValue::from_static(""));
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    h.insert("x-scene-id", hv(&meta.scene_id));
    h.insert("x-scene-date", hv(&meta.date.to_string()));
    h.insert("x-source", hv(&meta.source));
    h.insert("x-credit", hv(&meta.credit));
    h.insert("access-control-expose-headers", HeaderValue::from_static("x-scene-id, x-scene-date, x-source, x-credit"));
    Ok((h, png))
}

#[derive(Debug, Deserialize)]
pub struct LayerQuery { pub kind: String, pub lat: f64, pub lon: f64, pub size_m: Option<f64> }

/// GET /api/imagery/layer.png?kind=ndvi|slope&lat&lon&size_m → the coloured data layer for the dashboard map.
async fn layer_png(State(s): State<AppState>, Query(q): Query<LayerQuery>) -> ApiResult<impl IntoResponse> {
    let (png, _) = layer::layer(&s.http, &s.config.root, &q.kind, q.lat, q.lon, q.size_m.unwrap_or(1500.0)).await.map_err(upstream)?;
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    Ok((h, png))
}

/// GET /api/imagery/layer.json?kind&lat&lon&size_m → where to place the layer, its date, source and method.
async fn layer_json(State(s): State<AppState>, Query(q): Query<LayerQuery>) -> ApiResult<Json<layer::LayerMeta>> {
    let (_, meta) = layer::layer(&s.http, &s.config.root, &q.kind, q.lat, q.lon, q.size_m.unwrap_or(1500.0)).await.map_err(upstream)?;
    Ok(Json(meta))
}

async fn snapshot_json(State(s): State<AppState>, Query(q): Query<SnapshotQuery>) -> ApiResult<Json<SnapshotMeta>> {
    let (_, meta) = snapshot(&s, q.lat, q.lon, q.size_m.unwrap_or(1000.0), q.px.unwrap_or(768)).await.map_err(upstream)?;
    Ok(Json(meta))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_parsing_and_validation() {
        let r = parse_ring("-77.2107,-11.4706; -77.2093,-11.4706;-77.2093,-11.4694;-77.2107,-11.4694;-77.2107,-11.4706").unwrap();
        let f = make_field(None, r).unwrap();
        assert_eq!(f.ring.len(), 4);
        assert!((f.center.lat + 11.47).abs() < 1e-3);
        assert!(make_field(None, vec![]).is_err());
        assert!(make_field(Some(LatLon { lat: 95.0, lon: 0.0 }), vec![]).is_err());
        assert!(parse_ring("1;2").is_err());
    }
}

#[cfg(test)]
mod network_tests {
    //! Live tests against Planetary Computer for the Huaral pilot. Run: `cargo test -p imagery -- --ignored --nocapture`.
    use super::*;

    const LAT: f64 = -11.47;
    const LON: f64 = -77.21;

    fn huaral_2ha() -> Field {
        // ~141 m square = 2 ha.
        let h_lat = 70.7 / 111_320.0;
        let h_lon = 70.7 / (111_320.0 * LAT.to_radians().cos());
        Field {
            center: LatLon { lat: LAT, lon: LON },
            ring: vec![[LON - h_lon, LAT - h_lat], [LON + h_lon, LAT - h_lat], [LON + h_lon, LAT + h_lat], [LON - h_lon, LAT + h_lat]],
        }
    }

    fn http() -> reqwest::Client {
        reqwest::Client::builder().user_agent("FieldShift/0.1 (test)").timeout(std::time::Duration::from_secs(120)).build().unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn huaral_ndvi_series() {
        let f = huaral_2ha();
        println!("area {:.2} ha", f.area_ha());
        let t0 = std::time::Instant::now();
        let s = ndvi::ndvi_series_uncached(&http(), &f, &NdviParams::default()).await.unwrap();
        println!("took {:?}; scenes found {}, clear points {}, dropped {}", t0.elapsed(), s.scenes_found, s.points.len(), s.dropped.len());
        for p in &s.points {
            println!("{} {:<15} {:<6} mean {:.3} p10 {:?} p90 {:?} px {} cloud {}% {} {}", p.date, p.source, p.platform, p.mean, p.p10, p.p90, p.pixels, p.field_cloud_pct, p.scene_id, p.flag.clone().unwrap_or_default());
        }
        println!("latest {:?}", s.latest.as_ref().map(|p| (p.date, p.mean, &p.scene_id)));
        println!("bare soil {:#?}", s.bare_soil);
        println!("cycles {:#?}", s.cycles);
        for d in s.dropped.iter().take(10) { println!("dropped {} {} {}", d.date, d.scene_id, d.reason); }
        assert!(!s.points.is_empty());
        assert!(s.points.iter().any(|p| p.collection == Collection::LandsatC2L2));
        assert!(s.points.iter().any(|p| p.collection == Collection::Sentinel2L2a));
    }

    #[tokio::test]
    #[ignore]
    async fn huaral_point_ndvi() {
        let s = ndvi::ndvi_series_uncached(&http(), &Field::point(LAT, LON), &NdviParams { months: 6, ..Default::default() }).await.unwrap();
        println!("point: {} points, latest {:?}", s.points.len(), s.latest.as_ref().map(|p| (p.date, p.mean)));
        assert!(!s.points.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn huaral_terrain() {
        let t = terrain::terrain_uncached(&http(), &huaral_2ha()).await.unwrap();
        println!("{t:#?}");
        let p = terrain::terrain_uncached(&http(), &Field::point(LAT, LON)).await.unwrap();
        println!("point: elev {} slope mean {} max {} cells {}", p.elevation_mean_m, p.slope_mean_deg, p.slope_max_deg, p.cells);
        assert!(t.elevation_mean_m > 100.0 && t.elevation_mean_m < 300.0);
        assert!(t.slope_max_deg >= t.slope_mean_deg);
    }

    #[tokio::test]
    #[ignore]
    async fn huaral_snapshot() {
        let root = std::env::temp_dir().join("fieldshift_imagery_test");
        let (png, meta) = snapshot::snapshot(&http(), &root, LAT, LON, 1000.0, 512).await.unwrap();
        println!("{} bytes, {meta:#?}", png.len());
        assert!(png.starts_with(b"\x89PNG"));
        let (png2, _) = snapshot::snapshot(&http(), &root, LAT, LON, 1000.0, 512).await.unwrap();
        assert_eq!(png, png2, "second call comes from the disk cache");
    }

    #[tokio::test]
    #[ignore]
    async fn huaral_scene_search() {
        let to = Utc::now().date_naive();
        let from = to.checked_sub_months(Months::new(3)).unwrap();
        let s = stac::search_all(&http(), &huaral_2ha(), &[Collection::Sentinel2L2a, Collection::LandsatC2L2], from, to, 70.0).await.unwrap();
        for x in &s { println!("{} {} {} {:.1}%", x.date, x.source, x.id, x.scene_cloud_pct); }
        assert!(!s.is_empty());
    }
}
