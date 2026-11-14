//! Indicators for a period (temperature, heat and frost days, rain and extremes, ET0, water balance, GDD), each
//! compared with the same calendar window in the 1985-2024 normal.
use crate::{days::{slice, Day}, et0, gdd, meta::Meta, normals, stats::{round, NormalStat}, waterbalance};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HEAT_C: f64 = 32.0;
pub const FROST_C: f64 = 0.0;
pub const DEFAULT_GDD_BASE: f64 = 10.0;

/// User-tunable options. Defaults: Kc 1.0, heat threshold 32 °C, GDD base 10 °C, default season start.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Options {
    pub kc: f64,
    pub heat_c: f64,
    pub gdd_base: f64,
    /// Season start; `None` = 1 August (southern hemisphere) or 1 January (northern hemisphere).
    pub season_start: Option<NaiveDate>,
}

impl Default for Options {
    fn default() -> Self {
        Self { kc: waterbalance::DEFAULT_KC, heat_c: DEFAULT_HEAT_C, gdd_base: DEFAULT_GDD_BASE, season_start: None }
    }
}

/// One indicator value with its normal, anomaly and provenance.
#[derive(Clone, Debug, Serialize)]
pub struct Metric {
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub value: Option<f64>,
    /// Distribution of the same indicator over the same calendar dates in 1985-2024.
    pub normal: Option<NormalStat>,
    /// value - normal mean.
    pub anomaly: Option<f64>,
    pub meta: Meta,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeriodSummary {
    pub name: &'static str,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub days: usize,
    pub complete_days: usize,
    pub metrics: Vec<Metric>,
}

struct Def { key: &'static str, label: &'static str, unit: &'static str, method: String }

fn defs(o: &Options, p95: Option<f64>) -> Vec<Def> {
    let d = |key, label, unit, method: String| Def { key, label, unit, method };
    vec![
        d("tmean_c", "Mean temperature", "°C", "Mean of daily T2M".into()),
        d("tmax_mean_c", "Mean daily maximum", "°C", "Mean of daily T2M_MAX".into()),
        d("tmin_mean_c", "Mean daily minimum", "°C", "Mean of daily T2M_MIN".into()),
        d("heat_days", "Heat-stress days", "days", format!("Days with T2M_MAX ≥ {} °C (default 32 °C, overridable per crop)", o.heat_c)),
        d("frost_days", "Frost days", "days", format!("Days with T2M_MIN ≤ {FROST_C} °C")),
        d("rain_mm", "Rain total", "mm", "Sum of daily PRECTOTCORR".into()),
        d("rain_max_day_mm", "Wettest day", "mm", "Maximum daily PRECTOTCORR".into()),
        d("rain_extreme_days", "Extreme rain days", "days", match p95 {
            Some(p) => format!("Days with PRECTOTCORR ≥ {p} mm, the 95th percentile of wet days (≥ 1 mm) in 1985-2024"),
            None => "Unavailable: fewer than 30 wet days (≥ 1 mm) in 1985-2024, so no stable 95th percentile".into(),
        }),
        d("et0_mm", "Reference ET0", "mm", et0::METHOD.into()),
        d("water_balance_mm", "Water balance (rain - ET0×Kc)", "mm", format!("{} (Kc = {})", waterbalance::METHOD, o.kc)),
        d("gdd", "Growing degree days", "°C·day", format!("{} (base {} °C)", gdd::GDD_METHOD, o.gdd_base)),
    ]
}

/// Raw indicator values for a slice of days, in the order of `defs`.
pub fn values(s: &[Day], o: &Options, p95: Option<f64>) -> Vec<Option<f64>> {
    use normals::{mean_of, sum_of};
    let count = |f: &dyn Fn(&Day) -> Option<bool>| -> Option<f64> {
        let v: Vec<bool> = s.iter().filter_map(f).collect();
        (!v.is_empty()).then(|| v.iter().filter(|x| **x).count() as f64)
    };
    let both: Vec<&Day> = s.iter().filter(|d| d.rain.is_some() && d.et0.is_some()).collect();
    let wb = (!both.is_empty()).then(|| {
        let r: f64 = both.iter().map(|d| d.rain.unwrap()).sum();
        let e: f64 = both.iter().map(|d| d.et0.unwrap()).sum();
        waterbalance::balance(r, e, o.kc)
    });
    vec![
        mean_of(s, |d| d.t2m),
        mean_of(s, |d| d.tmax),
        mean_of(s, |d| d.tmin),
        count(&|d| d.tmax.map(|t| t >= o.heat_c)),
        count(&|d| d.tmin.map(|t| t <= FROST_C)),
        sum_of(s, |d| d.rain),
        s.iter().filter_map(|d| d.rain).reduce(f64::max),
        p95.and_then(|p| count(&|d| d.rain.map(|r| r >= p))),
        sum_of(s, |d| d.et0),
        wb,
        sum_of(s, |d| Some(gdd::gdd_avg(d.tmax?, d.tmin?, o.gdd_base))),
    ]
}

/// Indicators for `from..=to` plus their normals over the same calendar dates.
pub fn period(all: &[Day], name: &'static str, from: NaiveDate, to: NaiveDate, o: &Options, p95: Option<f64>) -> PeriodSummary {
    let s = slice(all, from, to);
    let now = values(s, o, p95);
    let years = normals::per_year(all, from, to, |w| values(w, o, p95));
    let period = format!("{from}..{to}");
    let metrics = defs(o, p95).into_iter().enumerate().map(|(k, def)| {
        let normal = normals::stat_of(&years, k);
        let value = now[k].map(|v| round(v, 2));
        let anomaly = match (value, &normal) { (Some(v), Some(n)) => Some(round(v - n.mean, 2)), _ => None };
        Metric {
            key: def.key, label: def.label, unit: def.unit, value, normal, anomaly,
            meta: Meta::new(period.clone(), format!("{}; normal = same calendar dates in {}", def.method, normals::WINDOW_LABEL)),
        }
    }).collect();
    PeriodSummary { name, from, to, days: s.len(), complete_days: s.iter().filter(|d| d.complete()).count(), metrics }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normals::tests::synthetic;

    #[test]
    fn synthetic_period_matches_its_normal() {
        let d = synthetic();
        let o = Options::default();
        let from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 1, 30).unwrap();
        let p = period(&d, "test", from, to, &o, Some(15.0));
        let get = |k: &str| p.metrics.iter().find(|m| m.key == k).unwrap();
        assert_eq!(p.days, 30);
        assert!(get("tmean_c").anomaly.unwrap().abs() < 1e-6);
        assert_eq!(get("et0_mm").value, Some(120.0));
        assert_eq!(get("water_balance_mm").value.unwrap(), get("rain_mm").value.unwrap() - 120.0);
        assert_eq!(get("frost_days").value, Some(0.0));
        assert!(get("gdd").value.unwrap() > 400.0);
        assert_eq!(get("tmean_c").normal.as_ref().unwrap().n_years, 40);
        assert_eq!(get("tmean_c").meta.badge, "regional context");
    }
}
