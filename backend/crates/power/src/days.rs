//! Row view of the daily series with derived ET0, plus calendar helpers (season start, same window in other years).
use crate::{client::PowerDaily, et0};
use chrono::{Datelike, Duration, NaiveDate};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Day {
    pub date: NaiveDate,
    pub t2m: Option<f64>,
    pub tmax: Option<f64>,
    pub tmin: Option<f64>,
    pub rain: Option<f64>,
    pub rh: Option<f64>,
    pub ws: Option<f64>,
    pub rs: Option<f64>,
    pub tdew: Option<f64>,
    pub ps: Option<f64>,
    pub et0: Option<f64>,
}

impl Day {
    /// True when every value the indicators use is present (temperatures, rain and ET0).
    pub fn complete(&self) -> bool {
        self.t2m.is_some() && self.tmax.is_some() && self.tmin.is_some() && self.rain.is_some() && self.et0.is_some()
    }
}

pub fn days(d: &PowerDaily) -> Vec<Day> {
    (0..d.len()).map(|i| {
        let date = d.date(i);
        let et0 = et0::et0_from_power(d.t2m_max[i], d.t2m_min[i], d.t2mdew[i], d.rh2m[i], d.ws2m[i],
            d.allsky_sfc_sw_dwn[i], d.ps[i], d.lat, date.ordinal(), d.elevation_m).map(|x| (x * 100.0).round() / 100.0);
        Day { date, t2m: d.t2m[i], tmax: d.t2m_max[i], tmin: d.t2m_min[i], rain: d.prectotcorr[i], rh: d.rh2m[i],
              ws: d.ws2m[i], rs: d.allsky_sfc_sw_dwn[i], tdew: d.t2mdew[i], ps: d.ps[i], et0 }
    }).collect()
}

/// Last date with complete data (POWER radiation lags temperature by a few days, so this is the common end).
pub fn last_complete(days: &[Day]) -> Option<NaiveDate> {
    days.iter().rev().find(|d| d.complete()).map(|d| d.date)
}

/// Slice of `days` covering `from..=to` (days are contiguous from days[0].date).
pub fn slice(days: &[Day], from: NaiveDate, to: NaiveDate) -> &[Day] {
    let Some(first) = days.first() else { return &[] };
    let a = (from - first.date).num_days().max(0) as usize;
    let b = ((to - first.date).num_days() + 1).max(0) as usize;
    let (a, b) = (a.min(days.len()), b.min(days.len()));
    if a >= b { &[] } else { &days[a..b] }
}

/// Month and day moved to `year`; 29 February becomes 28 February in non-leap years.
pub fn with_year(d: NaiveDate, year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, d.month(), d.day())
        .or_else(|| NaiveDate::from_ymd_opt(year, d.month(), 28))
        .unwrap()
}

/// The same calendar window (`from`, `len` days) in `year` for the year in which `from` falls.
pub fn same_window(from: NaiveDate, to: NaiveDate, year: i32) -> (NaiveDate, NaiveDate) {
    let s = with_year(from, year);
    (s, s + Duration::days((to - from).num_days()))
}

/// Default agricultural season start before `end`: 1 August in the southern hemisphere (Peru's campaign runs
/// August-July), 1 January in the northern hemisphere.
pub fn default_season_start(lat: f64, end: NaiveDate) -> NaiveDate {
    let (m, d) = if lat < 0.0 { (8, 1) } else { (1, 1) };
    let s = NaiveDate::from_ymd_opt(end.year(), m, d).unwrap();
    if s > end { NaiveDate::from_ymd_opt(end.year() - 1, m, d).unwrap() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_helpers() {
        let d = |y, m, dd| NaiveDate::from_ymd_opt(y, m, dd).unwrap();
        assert_eq!(with_year(d(2024, 2, 29), 2023), d(2023, 2, 28));
        assert_eq!(same_window(d(2026, 8, 1), d(2026, 11, 9), 1990), (d(1990, 8, 1), d(1990, 11, 9)));
        assert_eq!(same_window(d(2025, 12, 1), d(2026, 1, 15), 1990), (d(1990, 12, 1), d(1991, 1, 15)));
        assert_eq!(default_season_start(-11.47, d(2026, 11, 9)), d(2026, 8, 1));
        assert_eq!(default_season_start(-11.47, d(2026, 3, 9)), d(2025, 8, 1));
        assert_eq!(default_season_start(40.0, d(2026, 3, 9)), d(2026, 1, 1));
    }
}
