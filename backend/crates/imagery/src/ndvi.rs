//! NDVI time series for a field from Sentinel-2 L2A and Landsat C2 L2, per-field cloud screening, bare-soil periods and crop cycles.
use crate::{pc, stac::{self, Collection, Scene, BADGE}};
use chrono::{Months, NaiveDate, Utc};
use common::geo::{Field, LatLon};
use futures_util::{stream, StreamExt};
use serde::{Deserialize, Serialize};

/// Landsat Collection 2 Level-2 surface reflectance scale and offset (USGS).
pub const LANDSAT_SCALE: f64 = 0.0000275;
pub const LANDSAT_OFFSET: f64 = -0.2;
/// Sentinel-2 L2A: reflectance = (DN + BOA_ADD_OFFSET) / 10000, BOA_ADD_OFFSET = -1000 since baseline 04.00.
pub const S2_QUANT: f64 = 10_000.0;
pub const S2_BOA_OFFSET: f64 = -1000.0;

pub const METHOD: &str = "NDVI = (NIR - Red) / (NIR + Red) on surface reflectance, averaged over the field polygon \
(Sentinel-2 L2A B08/B04 at 10 m with the baseline-04.00 offset removed; Landsat 8/9 C2 L2 SR_B5/SR_B4 at 30 m, \
reflectance = DN x 0.0000275 - 0.2). Statistics by the Microsoft Planetary Computer data API over the field polygon. \
Scenes are kept only if the field itself is clear: Sentinel-2 SCL classes 3, 8, 9, 10 (shadow, cloud, cirrus) or \
Landsat QA_PIXEL without the 'clear' bit count as cloudy. Isolated drops of more than 0.25 between two close \
neighbours are flagged as probable haze or undetected cloud and left out of the period detection.";

/// NDVI from reflectances; None when the denominator vanishes.
pub fn ndvi(nir: f64, red: f64) -> Option<f64> {
    let d = nir + red;
    if d.abs() < 1e-9 { None } else { Some((nir - red) / d) }
}

/// Landsat C2 L2 surface reflectance from a digital number.
pub fn landsat_sr(dn: f64) -> f64 {
    dn * LANDSAT_SCALE + LANDSAT_OFFSET
}

/// Sentinel-2 L2A surface reflectance from a digital number.
pub fn s2_sr(dn: f64, has_offset: bool) -> f64 {
    (dn + if has_offset { S2_BOA_OFFSET } else { 0.0 }) / S2_QUANT
}

/// Band-math for the data API: `ndvi;cloud_fraction;nodata_fraction`, consistent with `s2_sr` / `landsat_sr`.
pub fn expression(scene: &Scene) -> String {
    match scene.collection {
        Collection::Sentinel2L2a => {
            // The offset cancels in the numerator and appears twice in the denominator.
            let ndvi = if scene.s2_has_offset() {
                format!("(B08-B04)/(B08+B04-{})", -2.0 * S2_BOA_OFFSET)
            } else {
                "(B08-B04)/(B08+B04)".to_string()
            };
            format!("{ndvi};where((SCL==3)|(SCL==8)|(SCL==9)|(SCL==10),1.0,0.0);where(SCL==0,1.0,0.0)")
        }
        Collection::LandsatC2L2 => {
            let sr = |b: &str| format!("({b}*{LANDSAT_SCALE}{LANDSAT_OFFSET:+})");
            let (n, r) = (sr("nir08"), sr("red"));
            // QA_PIXEL bit 6 = clear; bit 0 = fill (value 1).
            format!("({n}-{r})/({n}+{r});where((qa_pixel % 128) >= 64,0.0,1.0);where(qa_pixel==1,1.0,0.0)")
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NdviParams {
    pub months: u32,
    /// STAC pre-filter on whole-scene cloud cover (%). Coastal fog makes this loose on purpose: the field mask decides.
    pub max_scene_cloud_pct: f64,
    /// A scene is dropped when more than this share of the field is cloud or shadow (%).
    pub max_field_cloud_pct: f64,
    pub bare_soil_threshold: f64,
    pub bare_soil_min_obs: usize,
    /// Crop cycle: starts when NDVI reaches `cycle_high`, ends when it falls below `cycle_low`.
    pub cycle_high: f64,
    pub cycle_low: f64,
}

impl Default for NdviParams {
    fn default() -> Self {
        Self {
            months: 24,
            max_scene_cloud_pct: 70.0,
            max_field_cloud_pct: 10.0,
            bare_soil_threshold: 0.2,
            bare_soil_min_obs: 2,
            cycle_high: 0.45,
            cycle_low: 0.25,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NdviPoint {
    pub date: NaiveDate,
    pub source: String,
    pub collection: Collection,
    pub platform: String,
    pub scene_id: String,
    pub mean: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p10: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p90: Option<f64>,
    pub pixels: u32,
    pub field_cloud_pct: f64,
    pub scene_cloud_pct: f64,
    pub resolution_m: f64,
    pub badge: String,
    pub credit: String,
    /// "suspect_haze" when the value is an isolated drop (kept, but not used for periods).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DroppedScene {
    pub scene_id: String,
    pub date: NaiveDate,
    pub source: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BareSoilPeriod {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub days: i64,
    pub observations: usize,
    pub mean_ndvi: f64,
    pub min_ndvi: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CropCycle {
    /// Last observation below `cycle_low` before green-up (None if the series starts mid-cycle).
    pub start: Option<NaiveDate>,
    pub peak_date: NaiveDate,
    pub peak_ndvi: f64,
    /// First observation below `cycle_low` after the peak (None if still green).
    pub end: Option<NaiveDate>,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldInfo {
    pub center: LatLon,
    /// "polygon" or "point-box" (a ~110 m square around the point).
    pub kind: String,
    pub area_ha: f64,
    pub polygon: Vec<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NdviSeries {
    pub field: FieldInfo,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub params: NdviParams,
    pub latest: Option<NdviPoint>,
    /// Clear observations, oldest first.
    pub points: Vec<NdviPoint>,
    pub bare_soil: Vec<BareSoilPeriod>,
    pub cycles: Vec<CropCycle>,
    pub dropped: Vec<DroppedScene>,
    pub scenes_found: usize,
    pub badge: String,
    pub sources: Vec<String>,
    pub credits: Vec<String>,
    pub method: String,
    pub notes: Vec<String>,
    pub generated_at: String,
}

/// Runs of at least `min_obs` consecutive observations with NDVI below `threshold`.
pub fn detect_bare_soil(series: &[(NaiveDate, f64)], threshold: f64, min_obs: usize) -> Vec<BareSoilPeriod> {
    let mut out = Vec::new();
    let mut run: Vec<(NaiveDate, f64)> = Vec::new();
    let flush = |run: &mut Vec<(NaiveDate, f64)>, out: &mut Vec<BareSoilPeriod>| {
        if run.len() >= min_obs.max(1) {
            let (start, end) = (run[0].0, run[run.len() - 1].0);
            out.push(BareSoilPeriod {
                start,
                end,
                days: (end - start).num_days(),
                observations: run.len(),
                mean_ndvi: round3(run.iter().map(|p| p.1).sum::<f64>() / run.len() as f64),
                min_ndvi: round3(run.iter().map(|p| p.1).fold(f64::MAX, f64::min)),
            });
        }
        run.clear();
    };
    for &(d, v) in series {
        if v < threshold {
            run.push((d, v));
        } else {
            flush(&mut run, &mut out);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// Crop cycles by hysteresis: green-up when NDVI reaches `high`, senescence/harvest when it drops below `low`.
pub fn detect_cycles(series: &[(NaiveDate, f64)], high: f64, low: f64) -> Vec<CropCycle> {
    let mut out = Vec::new();
    let mut last_low: Option<NaiveDate> = None;
    let mut cur: Option<CropCycle> = None;
    for &(d, v) in series {
        match cur.as_mut() {
            None => {
                if v < low {
                    last_low = Some(d);
                }
                if v >= high {
                    cur = Some(CropCycle { start: last_low, peak_date: d, peak_ndvi: round3(v), end: None, complete: false });
                }
            }
            Some(c) => {
                if v > c.peak_ndvi {
                    c.peak_ndvi = round3(v);
                    c.peak_date = d;
                }
                if v < low {
                    c.end = Some(d);
                    c.complete = c.start.is_some();
                    out.push(cur.take().unwrap());
                    last_low = Some(d);
                }
            }
        }
    }
    out.extend(cur);
    out
}

/// Indices of isolated drops (> `min_drop` below both neighbours, neighbours at most `max_gap_days` apart).
pub fn spike_indices(series: &[(NaiveDate, f64)], min_drop: f64, max_gap_days: i64) -> Vec<usize> {
    (1..series.len().saturating_sub(1))
        .filter(|&i| {
            let (a, b, c) = (series[i - 1], series[i], series[i + 1]);
            b.1 < a.1 - min_drop && b.1 < c.1 - min_drop && (c.0 - a.0).num_days() <= max_gap_days
        })
        .collect()
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

enum Outcome {
    Point(NdviPoint),
    Dropped(DroppedScene),
}

async fn scene_ndvi(http: &reqwest::Client, scene: &Scene, feature: &serde_json::Value, params: &NdviParams) -> Outcome {
    let drop = |reason: String| {
        Outcome::Dropped(DroppedScene { scene_id: scene.id.clone(), date: scene.date, source: scene.source.clone(), reason })
    };
    let stats = match pc::statistics(http, scene.collection.id(), &scene.id, &expression(scene), feature, true).await {
        Ok(s) if s.len() == 3 => s,
        Ok(_) => return drop("data API returned an unexpected band count".into()),
        Err(e) => return drop(format!("data API error: {e}")),
    };
    let (nd, cloud, nodata) = (&stats[0], stats[1].mean * 100.0, stats[2].mean * 100.0);
    if !(nodata < 50.0) || stats[0].valid_pixels < 1.0 {
        return drop("no data over the field (scene edge)".into());
    }
    if !(cloud <= params.max_field_cloud_pct) {
        return drop(format!("cloud or shadow over {:.0}% of the field", cloud));
    }
    if !nd.mean.is_finite() || nd.mean < -1.0 || nd.mean > 1.0 {
        return drop("invalid NDVI".into());
    }
    Outcome::Point(NdviPoint {
        date: scene.date,
        source: scene.source.clone(),
        collection: scene.collection,
        platform: scene.platform.clone(),
        scene_id: scene.id.clone(),
        mean: round3(nd.mean),
        p10: nd.p10.filter(|v| v.is_finite()).map(round3),
        p90: nd.p90.filter(|v| v.is_finite()).map(round3),
        pixels: nd.valid_pixels as u32,
        field_cloud_pct: (cloud * 10.0).round() / 10.0,
        scene_cloud_pct: (scene.scene_cloud_pct * 10.0).round() / 10.0,
        resolution_m: scene.resolution_m,
        badge: BADGE.into(),
        credit: scene.credit.clone(),
        flag: None,
    })
}

pub fn field_info(field: &Field) -> FieldInfo {
    let poly = pc::field_as_polygon(field);
    FieldInfo {
        center: field.center,
        kind: if field.ring.len() >= 3 { "polygon" } else { "point-box" }.into(),
        area_ha: (poly.area_ha() * 100.0).round() / 100.0,
        polygon: poly.ring,
    }
}

/// Computes the NDVI series straight from Planetary Computer (no database cache).
pub async fn ndvi_series_uncached(http: &reqwest::Client, field: &Field, params: &NdviParams) -> anyhow::Result<NdviSeries> {
    let to = Utc::now().date_naive();
    let from = to.checked_sub_months(Months::new(params.months.clamp(1, 120))).unwrap_or(to);
    let collections = [Collection::Sentinel2L2a, Collection::LandsatC2L2];
    let scenes = stac::search_all(http, field, &collections, from, to, params.max_scene_cloud_pct).await?;
    let feature = pc::feature(&pc::field_ring(field));
    let outcomes: Vec<Outcome> = stream::iter(scenes.iter().cloned())
        .map(|s| {
            let (http, feature, params) = (http.clone(), feature.clone(), params.clone());
            async move { scene_ndvi(&http, &s, &feature, &params).await }
        })
        .buffer_unordered(8)
        .collect()
        .await;

    let mut points = Vec::new();
    let mut dropped = Vec::new();
    for o in outcomes {
        match o {
            Outcome::Point(p) => points.push(p),
            Outcome::Dropped(d) => dropped.push(d),
        }
    }
    // Same sensor, same day (tile overlap): keep the clearest copy.
    points.sort_by(|a, b| {
        (a.date, a.collection as u8).cmp(&(b.date, b.collection as u8))
            .then(a.field_cloud_pct.total_cmp(&b.field_cloud_pct))
    });
    points.dedup_by(|b, a| a.date == b.date && a.collection == b.collection);

    let all: Vec<(NaiveDate, f64)> = points.iter().map(|p| (p.date, p.mean)).collect();
    for i in spike_indices(&all, 0.25, 40) {
        points[i].flag = Some("suspect_haze".into());
    }
    let clean: Vec<(NaiveDate, f64)> = points.iter().filter(|p| p.flag.is_none()).map(|p| (p.date, p.mean)).collect();
    let bare_soil = detect_bare_soil(&clean, params.bare_soil_threshold, params.bare_soil_min_obs);
    let cycles = detect_cycles(&clean, params.cycle_high, params.cycle_low);
    dropped.sort_by(|a, b| b.date.cmp(&a.date));

    let mut sources: Vec<String> = points.iter().map(|p| p.source.clone()).collect();
    sources.sort();
    sources.dedup();
    let mut credits: Vec<String> = points.iter().map(|p| p.credit.clone()).collect();
    credits.sort();
    credits.dedup();
    let mut notes = vec![format!(
        "Bare soil: NDVI below {} on at least {} consecutive clear observations; crop cycle: NDVI rises to {} and later falls below {}.",
        params.bare_soil_threshold, params.bare_soil_min_obs, params.cycle_high, params.cycle_low
    )];
    if field.ring.len() < 3 {
        notes.push("Only a point was given: statistics use a ~110 m square around it, which may mix neighbouring plots.".into());
    }
    notes.push("Sentinel-2 and Landsat values are merged without cross-sensor harmonisation (differences are usually under 0.05 NDVI).".into());

    Ok(NdviSeries {
        field: field_info(field),
        from,
        to,
        params: params.clone(),
        latest: points.iter().rev().find(|p| p.flag.is_none()).cloned(),
        points,
        bare_soil,
        cycles,
        dropped,
        scenes_found: scenes.len(),
        badge: BADGE.into(),
        sources,
        credits,
        method: METHOD.into(),
        notes,
        generated_at: Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn ndvi_math() {
        assert!((ndvi(0.5, 0.1).unwrap() - 0.6666667).abs() < 1e-6);
        assert_eq!(ndvi(0.0, 0.0), None);
        assert!((ndvi(0.1, 0.3).unwrap() + 0.5).abs() < 1e-12);
    }

    #[test]
    fn landsat_scaling() {
        assert!((landsat_sr(7273.0) - 0.0000075).abs() < 1e-6);
        assert!((landsat_sr(43636.0) - 1.00).abs() < 1e-3);
        // NDVI from DNs equals the expression's closed form.
        let (nir, red) = (21000.0, 9500.0);
        let v = ndvi(landsat_sr(nir), landsat_sr(red)).unwrap();
        let k = LANDSAT_SCALE;
        let closed = ((nir * k - 0.2) - (red * k - 0.2)) / ((nir * k - 0.2) + (red * k - 0.2));
        assert!((v - closed).abs() < 1e-12);
    }

    #[test]
    fn s2_offset_cancels_as_in_expression() {
        let (nir, red) = (3055.0, 2049.0);
        let v = ndvi(s2_sr(nir, true), s2_sr(red, true)).unwrap();
        assert!((v - (nir - red) / (nir + red - 2000.0)).abs() < 1e-12);
        assert!((s2_sr(1000.0, true)).abs() < 1e-12);
        assert!((s2_sr(1000.0, false) - 0.1).abs() < 1e-12);
    }

    #[test]
    fn landsat_expression_text() {
        let s = Scene {
            id: "L".into(), collection: Collection::LandsatC2L2, source: String::new(), platform: String::new(),
            datetime: String::new(), date: d("2026-10-09"), scene_cloud_pct: 0.0, resolution_m: 30.0,
            processing_baseline: None, credit: String::new(),
        };
        let e = expression(&s);
        assert!(e.starts_with("((nir08*0.0000275-0.2)-(red*0.0000275-0.2))/((nir08*0.0000275-0.2)+(red*0.0000275-0.2));"), "{e}");
        assert_eq!(e.split(';').count(), 3);
    }

    #[test]
    fn bare_soil_needs_two_consecutive() {
        let s = vec![
            (d("2025-01-01"), 0.6),
            (d("2025-01-11"), 0.15), // single low: not a period
            (d("2025-01-21"), 0.5),
            (d("2025-02-01"), 0.12),
            (d("2025-02-11"), 0.10),
            (d("2025-02-21"), 0.18),
            (d("2025-03-01"), 0.35),
            (d("2025-03-11"), 0.19),
            (d("2025-03-21"), 0.11), // run at the end counts
        ];
        let p = detect_bare_soil(&s, 0.2, 2);
        assert_eq!(p.len(), 2);
        assert_eq!((p[0].start, p[0].end, p[0].observations, p[0].days), (d("2025-02-01"), d("2025-02-21"), 3, 20));
        assert_eq!(p[0].min_ndvi, 0.1);
        assert_eq!((p[1].start, p[1].end), (d("2025-03-11"), d("2025-03-21")));
        assert!(detect_bare_soil(&s, 0.2, 4).is_empty());
    }

    #[test]
    fn cycles_by_hysteresis() {
        let s = vec![
            (d("2025-01-01"), 0.15),
            (d("2025-02-01"), 0.30),
            (d("2025-03-01"), 0.60),
            (d("2025-04-01"), 0.75),
            (d("2025-05-01"), 0.40),
            (d("2025-06-01"), 0.18),
            (d("2025-07-01"), 0.50),
        ];
        let c = detect_cycles(&s, 0.45, 0.25);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], CropCycle { start: Some(d("2025-01-01")), peak_date: d("2025-04-01"), peak_ndvi: 0.75, end: Some(d("2025-06-01")), complete: true });
        assert_eq!((c[1].start, c[1].end, c[1].complete), (Some(d("2025-06-01")), None, false));
    }

    #[test]
    fn spikes_are_isolated_drops() {
        let s = vec![(d("2025-01-01"), 0.6), (d("2025-01-11"), 0.2), (d("2025-01-21"), 0.62), (d("2025-03-01"), 0.1), (d("2025-05-01"), 0.6)];
        assert_eq!(spike_indices(&s, 0.25, 40), vec![1]);
    }
}
