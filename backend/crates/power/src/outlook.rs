//! Season outlook: an ESTIMATE (never a forecast) for the next 1–6 months from NASA POWER alone.
//! Two scenarios per month: the 1985–2024 normal for that month, and "the current 30-day anomaly persists".
//! Pest generations to the end of the window under both scenarios, single-sine degree-days on the same years.
use crate::{days, gdd, normals, pests, PowerDaily};
use chrono::{Datelike, Duration, NaiveDate};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Bi { pub en: String, pub es: String }

#[derive(Clone, Debug, Serialize)]
pub struct OutlookMonth { pub month: String, pub normal_mean: f64, pub normal_lo: f64, pub normal_hi: f64, pub persist_mean: f64 }

#[derive(Clone, Debug, Serialize)]
pub struct OutlookPest { pub name: Bi, pub scientific: String, pub to_date: f64, pub by_end_normal: f64, pub by_end_persist: f64 }

#[derive(Clone, Debug, Serialize)]
pub struct Outlook {
    pub label: Bi,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub anomaly_c: f64,
    pub months: Vec<OutlookMonth>,
    pub pests: Vec<OutlookPest>,
    pub source: String,
    pub method: String,
    pub badge: String,
}

fn r1(x: f64) -> f64 { (x * 10.0).round() / 10.0 }

fn add_months(d: NaiveDate, m: u32) -> NaiveDate {
    let total = d.year() * 12 + d.month0() as i32 + m as i32;
    NaiveDate::from_ymd_opt(total / 12, (total % 12) as u32 + 1, 1).unwrap()
}

/// A pest to project: published lower threshold (and upper cutoff, if any) and degree-days per generation.
#[derive(Clone, Debug)]
pub struct PestModel { pub name: Bi, pub scientific: String, pub lower_c: f64, pub upper_c: Option<f64>, pub dd_per_generation: f64 }

/// The general reference pests (used only when no crop is known).
pub fn reference_models() -> Vec<PestModel> {
    pests::table().into_iter().map(|p| PestModel {
        name: Bi { en: p.common_name.into(), es: crate::pests::name_es(p.key).into() }, scientific: p.scientific_name.into(),
        lower_c: p.lower_c, upper_c: p.upper_c, dd_per_generation: p.dd_per_generation }).collect()
}

pub fn build(d: &PowerDaily, months: u32) -> anyhow::Result<Outlook> { build_with(d, months, &reference_models()) }

/// The outlook with the given pests (the field's own crop pests when known).
pub fn build_with(d: &PowerDaily, months: u32, models: &[PestModel]) -> anyhow::Result<Outlook> {
    let months = months.clamp(1, 6);
    let all = days::days(d);
    let end = days::last_complete(&all).ok_or_else(|| anyhow::anyhow!("no complete POWER days"))?;
    let start = end + Duration::days(1);
    let to = add_months(start, months) - Duration::days(1);
    // Current anomaly: last 30 days vs the same dates in 1985-2024.
    let recent: Vec<f64> = days::slice(&all, end - Duration::days(29), end).iter().filter_map(|x| x.t2m).collect();
    let normal_recent: Vec<f64> = (1985..=2024).filter_map(|y| {
        let (f, e) = days::same_window(end - Duration::days(29), end, y);
        let v: Vec<f64> = days::slice(&all, f, e).iter().filter_map(|x| x.t2m).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    }).collect();
    anyhow::ensure!(!recent.is_empty() && !normal_recent.is_empty(), "not enough data for the anomaly");
    let anomaly = recent.iter().sum::<f64>() / recent.len() as f64 - normal_recent.iter().sum::<f64>() / normal_recent.len() as f64;

    let mn = normals::monthly_normals(&all);
    let mut out_months = vec![];
    // Months after the current one (the current month already appears in the past-12-months chart).
    let mut m = add_months(start, 1);
    while m <= to {
        if let Some(n) = mn.iter().find(|x| x.month == m.month()).and_then(|x| x.tmean_c.clone()) {
            out_months.push(OutlookMonth { month: format!("{}-{:02}", m.year(), m.month()), normal_mean: r1(n.mean), normal_lo: r1(n.p10), normal_hi: r1(n.p90), persist_mean: r1(n.mean + anomaly) });
        }
        m = add_months(m, 1);
    }

    // Pests: generations so far this season, then the remaining window averaged over 1985-2024 (normal) and shifted by the anomaly (persist).
    let season_start = days::default_season_start(d_lat(d), end);
    let mut out_pests = vec![];
    for p in models {
        let gen = |from: NaiveDate, to: NaiveDate, shift: f64, year: Option<i32>| -> Option<f64> {
            let (f, t) = match year { Some(y) => days::same_window(from, to, y), None => (from, to) };
            let s = days::slice(&all, f, t);
            if s.is_empty() { return None; }
            Some(s.iter().filter_map(|x| Some(gdd::dd_sine(x.tmin? + shift, x.tmax? + shift, p.lower_c, p.upper_c))).sum::<f64>() / p.dd_per_generation)
        };
        let Some(so_far) = gen(season_start, end, 0.0, None) else { continue };
        let avg = |shift: f64| -> f64 {
            let v: Vec<f64> = (1985..=2024).filter_map(|y| gen(start, to, shift, Some(y))).collect();
            if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
        };
        out_pests.push(OutlookPest {
            name: p.name.clone(),
            scientific: p.scientific.clone(),
            to_date: r1(so_far), by_end_normal: r1(so_far + avg(0.0)), by_end_persist: r1(so_far + avg(anomaly)),
        });
    }
    Ok(Outlook {
        label: Bi { en: "Estimate, not a forecast".into(), es: "Estimación, no pronóstico".into() },
        from: start, to, anomaly_c: r1(anomaly), months: out_months, pests: out_pests,
        source: "NASA POWER (MERRA-2 / CERES SYN1deg) normals 1985–2024".into(),
        method: format!("Two scenarios for {start}..{to}: the 1985–2024 normal, and the normal shifted by the last 30 days' anomaly ({:+.1} °C). Pest generations: single-sine degree-days summed over the same calendar window in each normal year (and shifted for the warm scenario), added to this season's generations so far.", anomaly),
        badge: "regional context".into(),
    })
}

fn d_lat(d: &PowerDaily) -> f64 { d.lat }
