//! 40-year normals computed from the POWER daily series (window 1985-2024: 40 complete years, all parameters present;
//! POWER all-sky radiation starts in mid-1983, so 1981-1984 cannot feed ET0). Normals are built per calendar window
//! (same dates in every normal year), per calendar month and per day of year (±7-day moving window).
use crate::{days::{same_window, slice, Day}, stats::{mean, percentile, round, NormalStat}};
use chrono::{Datelike, NaiveDate};
use serde::Serialize;

pub const START_YEAR: i32 = 1985;
pub const END_YEAR: i32 = 2024;
pub const WINDOW_LABEL: &str = "1985-2024 (40 years)";
/// A normal year counts only if at least this share of its window days is complete.
pub const MIN_COVERAGE: f64 = 0.9;
/// Wet day threshold for the rain percentile (ETCCDI convention, 1 mm).
pub const WET_DAY_MM: f64 = 1.0;
/// Fewer wet days than this in the normal window and the 95th percentile is reported as unavailable.
pub const MIN_WET_DAYS: usize = 30;

/// Applies `f` to the same calendar window in each normal year and returns one value per usable year.
pub fn per_year<F>(days: &[Day], from: NaiveDate, to: NaiveDate, f: F) -> Vec<Vec<Option<f64>>>
where
    F: Fn(&[Day]) -> Vec<Option<f64>>,
{
    let mut out = vec![];
    for y in START_YEAR..=END_YEAR {
        let (a, b) = same_window(from, to, y);
        let s = slice(days, a, b);
        let want = (b - a).num_days() as usize + 1;
        if s.len() < want { continue; }
        let complete = s.iter().filter(|d| d.complete()).count();
        if (complete as f64) < MIN_COVERAGE * want as f64 { continue; }
        out.push(f(s));
    }
    out
}

/// Normal distribution of indicator `k` from `per_year` output.
pub fn stat_of(years: &[Vec<Option<f64>>], k: usize) -> Option<NormalStat> {
    let v: Vec<f64> = years.iter().filter_map(|row| row.get(k).copied().flatten()).collect();
    NormalStat::from(&v)
}

/// 95th percentile of daily rain on wet days (≥ 1 mm) over the normal window, if there are enough wet days.
pub fn rain_p95(days: &[Day]) -> (Option<f64>, usize) {
    let wet: Vec<f64> = days.iter()
        .filter(|d| (START_YEAR..=END_YEAR).contains(&d.date.year()))
        .filter_map(|d| d.rain).filter(|r| *r >= WET_DAY_MM).collect();
    if wet.len() < MIN_WET_DAYS { return (None, wet.len()); }
    (percentile(&wet, 95.0).map(|x| round(x, 2)), wet.len())
}

/// Normal of one calendar month: distribution across years of the monthly mean temperatures and totals.
#[derive(Clone, Debug, Serialize)]
pub struct MonthNormal {
    pub month: u32,
    pub tmean_c: Option<NormalStat>,
    pub tmax_c: Option<NormalStat>,
    pub tmin_c: Option<NormalStat>,
    pub rain_mm: Option<NormalStat>,
    pub et0_mm: Option<NormalStat>,
}

pub fn monthly_normals(days: &[Day]) -> Vec<MonthNormal> {
    (1..=12).map(|m| {
        let from = NaiveDate::from_ymd_opt(2001, m, 1).unwrap();
        let to = last_of_month(from);
        let years = per_year(days, from, to, |s| vec![
            mean_of(s, |d| d.t2m), mean_of(s, |d| d.tmax), mean_of(s, |d| d.tmin), sum_of(s, |d| d.rain), sum_of(s, |d| d.et0),
        ]);
        MonthNormal {
            month: m,
            tmean_c: stat_of(&years, 0),
            tmax_c: stat_of(&years, 1),
            tmin_c: stat_of(&years, 2),
            rain_mm: stat_of(&years, 3),
            et0_mm: stat_of(&years, 4),
        }
    }).collect()
}

/// Day-of-year normal (mean over 1985-2024 of a ±7-day window around the same calendar day).
#[derive(Clone, Debug, Serialize)]
pub struct DoyNormal {
    pub t2m: Option<f64>,
    pub tmax: Option<f64>,
    pub tmin: Option<f64>,
    pub rain: Option<f64>,
    pub et0: Option<f64>,
}

/// 365 slots indexed by `doy_index` (29 February shares the 28 February slot).
pub fn doy_normals(days: &[Day]) -> Vec<DoyNormal> {
    let mut acc = vec![[(0.0f64, 0usize); 5]; 365];
    for d in days.iter().filter(|d| (START_YEAR..=END_YEAR).contains(&d.date.year())) {
        let i = doy_index(d.date);
        for (k, v) in [d.t2m, d.tmax, d.tmin, d.rain, d.et0].into_iter().enumerate() {
            if let Some(v) = v { acc[i][k].0 += v; acc[i][k].1 += 1; }
        }
    }
    (0..365).map(|i| {
        let w = |k: usize| -> Option<f64> {
            let (mut s, mut n) = (0.0, 0usize);
            for o in -7i32..=7 {
                let j = (i as i32 + o).rem_euclid(365) as usize;
                s += acc[j][k].0;
                n += acc[j][k].1;
            }
            (n > 0).then(|| round(s / n as f64, 2))
        };
        DoyNormal { t2m: w(0), tmax: w(1), tmin: w(2), rain: w(3), et0: w(4) }
    }).collect()
}

/// 0-based day of a 365-day year; 29 February maps to 28 February.
pub fn doy_index(d: NaiveDate) -> usize {
    let n = NaiveDate::from_ymd_opt(2001, d.month(), if d.month() == 2 && d.day() == 29 { 28 } else { d.day() }).unwrap();
    n.ordinal0() as usize
}

pub fn last_of_month(d: NaiveDate) -> NaiveDate {
    let (y, m) = if d.month() == 12 { (d.year() + 1, 1) } else { (d.year(), d.month() + 1) };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap().pred_opt().unwrap()
}

pub fn mean_of(s: &[Day], f: impl Fn(&Day) -> Option<f64>) -> Option<f64> {
    let v: Vec<f64> = s.iter().filter_map(f).collect();
    mean(&v)
}

pub fn sum_of(s: &[Day], f: impl Fn(&Day) -> Option<f64>) -> Option<f64> {
    let v: Vec<f64> = s.iter().filter_map(f).collect();
    (!v.is_empty()).then(|| v.iter().sum())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::Duration;

    /// Synthetic series 1981..2026 with a sinusoidal annual cycle, constant rain and fixed ET0.
    pub fn synthetic() -> Vec<Day> {
        let start = NaiveDate::from_ymd_opt(1981, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        let n = (end - start).num_days() + 1;
        (0..n).map(|i| {
            let date = start + Duration::days(i);
            let t = 20.0 + 5.0 * (2.0 * std::f64::consts::PI * date.ordinal0() as f64 / 365.0).cos();
            let wet = date.ordinal0() % 10 == 0;
            Day { date, t2m: Some(t), tmax: Some(t + 4.0), tmin: Some(t - 4.0), rain: Some(if wet { 10.0 + (date.year() % 7) as f64 } else { 0.0 }),
                  rh: Some(70.0), ws: Some(2.0), rs: Some(20.0), tdew: None, ps: None, et0: Some(4.0) }
        }).collect()
    }

    #[test]
    fn monthly_normals_from_synthetic() {
        let d = synthetic();
        let m = monthly_normals(&d);
        assert_eq!(m.len(), 12);
        let jan = m[0].tmean_c.as_ref().unwrap();
        assert_eq!(jan.n_years, 40);
        assert!(jan.mean > 24.0 && jan.mean <= 25.0, "{}", jan.mean);
        assert!(jan.sd.unwrap() < 1e-6);
        let jul = m[6].tmean_c.as_ref().unwrap();
        assert!(jul.mean < 16.0, "{}", jul.mean);
        assert_eq!(m[0].et0_mm.as_ref().unwrap().mean, 124.0);
    }

    #[test]
    fn doy_and_p95() {
        let d = synthetic();
        let doy = doy_normals(&d);
        assert_eq!(doy.len(), 365);
        assert!((doy[0].t2m.unwrap() - 24.9).abs() < 0.2, "{:?}", doy[0].t2m);
        let (p95, n) = rain_p95(&d);
        assert!(n > 1000);
        assert!(p95.unwrap() >= 15.0 && p95.unwrap() <= 16.0, "{p95:?}");
        assert_eq!(doy_index(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()), 58);
        assert_eq!(doy_index(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()), 364);
    }
}
