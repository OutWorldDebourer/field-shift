//! Pure builders for the three responses (summary, daily series, monthly vs normal) from a `PowerDaily`.
use crate::{
    client::{request_url, PowerDaily, FIRST_DAY},
    days::{self, default_season_start, Day},
    et0, indicators::{self, Options, PeriodSummary}, meta::{self, Meta}, normals::{self, MonthNormal}, pests::{self, PestGenerations},
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    /// Elevation of the POWER grid cell (not of the field), m.
    pub cell_elevation_m: f64,
    pub note: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Attribution {
    pub source: &'static str,
    pub dataset: &'static str,
    pub resolution: &'static str,
    pub badge: &'static str,
    pub license: &'static str,
    pub upstream_sources: Vec<String>,
    pub api_version: String,
    pub request: String,
    pub fetched_at: DateTime<Utc>,
    pub citation: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct NormalsWindow {
    pub start_year: i32,
    pub end_year: i32,
    pub label: &'static str,
    pub method: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct PowerSummary {
    pub location: Location,
    /// Last day with complete data (radiation lags temperature by ~2 days, POWER by ~3-5 days overall).
    pub data_through: NaiveDate,
    pub options: Options,
    pub normals_window: NormalsWindow,
    pub rain_p95_mm: Option<f64>,
    pub wet_days_in_normal: usize,
    /// last_30_days, last_90_days, season_to_date.
    pub periods: Vec<PeriodSummary>,
    pub pests: Vec<PestGenerations>,
    pub monthly_normals: Vec<MonthNormal>,
    pub monthly_normals_meta: Meta,
    pub attribution: Attribution,
    pub notes: Vec<&'static str>,
}

const CELL_NOTE: &str = "POWER values describe a ~50 km grid cell that can mix ocean, valley and hills: regional context, not the field itself";
const NORMALS_METHOD: &str = "Computed by Field Shift from the POWER daily series: for each indicator, the same calendar dates in each of 1985-2024 (years with ≥ 90 % complete days); mean, sample sd, p10 and p90 across years";
const CITATION: &str = "NASA Langley Research Center (LaRC) POWER Project, funded through the NASA Earth Science/Applied Science Program; data obtained from the POWER API, https://power.larc.nasa.gov";

pub fn location(d: &PowerDaily) -> Location {
    Location { lat: d.lat, lon: d.lon, cell_elevation_m: d.elevation_m, note: CELL_NOTE }
}

pub fn attribution(d: &PowerDaily) -> Attribution {
    let start = NaiveDate::from_ymd_opt(FIRST_DAY.0, FIRST_DAY.1, FIRST_DAY.2).unwrap();
    let end = d.date(d.len().saturating_sub(1));
    Attribution {
        source: meta::SOURCE, dataset: meta::DATASET, resolution: meta::RESOLUTION, badge: meta::BADGE, license: meta::LICENSE,
        upstream_sources: d.sources.clone(), api_version: d.api_version.clone(),
        request: request_url(d.lat, d.lon, start, end), fetched_at: d.fetched_at, citation: CITATION,
    }
}

fn normals_window() -> NormalsWindow {
    NormalsWindow { start_year: normals::START_YEAR, end_year: normals::END_YEAR, label: normals::WINDOW_LABEL, method: NORMALS_METHOD }
}

fn end_of(all: &[Day]) -> anyhow::Result<NaiveDate> {
    days::last_complete(all).ok_or_else(|| anyhow::anyhow!("POWER series has no complete day"))
}

pub fn build_summary(d: &PowerDaily, opts: &Options) -> anyhow::Result<PowerSummary> {
    let all = days::days(d);
    let end = end_of(&all)?;
    let mut o = opts.clone();
    let season_start = o.season_start.filter(|s| *s <= end).unwrap_or_else(|| default_season_start(d.lat, end));
    o.season_start = Some(season_start);
    let (p95, wet) = normals::rain_p95(&all);
    let periods = vec![
        indicators::period(&all, "last_30_days", end - Duration::days(29), end, &o, p95),
        indicators::period(&all, "last_90_days", end - Duration::days(89), end, &o, p95),
        indicators::period(&all, "season_to_date", season_start, end, &o, p95),
    ];
    Ok(PowerSummary {
        location: location(d),
        data_through: end,
        normals_window: normals_window(),
        rain_p95_mm: p95,
        wet_days_in_normal: wet,
        periods,
        pests: pests::generations(&all, season_start, end),
        monthly_normals: normals::monthly_normals(&all),
        monthly_normals_meta: Meta::new(normals::WINDOW_LABEL, format!("Per calendar month: monthly mean of T2M / T2M_MAX / T2M_MIN, totals of PRECTOTCORR and ET0 ({}); {NORMALS_METHOD}", et0::METHOD)),
        attribution: attribution(d),
        options: o,
        notes: vec![
            "Regional context: POWER is a ~50 km reanalysis/satellite product, not a field measurement.",
            "Irrigation is not modelled in the water balance; a negative balance is the demand irrigation must cover.",
            "Pest generations are degree-day estimates from published laboratory thresholds; they indicate pressure, not observed populations.",
            "Season start defaults to 1 August (southern hemisphere, Peru's August-July campaign) or 1 January (northern hemisphere).",
        ],
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct Column {
    pub key: &'static str,
    pub unit: &'static str,
    pub meta: Meta,
}

#[derive(Clone, Debug, Serialize)]
pub struct SeriesRow {
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
    pub normal_t2m: Option<f64>,
    pub normal_tmax: Option<f64>,
    pub normal_tmin: Option<f64>,
    pub normal_rain: Option<f64>,
    pub normal_et0: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SeriesResponse {
    pub location: Location,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub data_through: NaiveDate,
    pub columns: Vec<Column>,
    pub rows: Vec<SeriesRow>,
    pub attribution: Attribution,
}

pub const SERIES_COLUMNS: [(&str, &str, &str); 16] = [
    ("date", "", "Local solar day (POWER time standard LST)"),
    ("t2m", "°C", "POWER T2M, temperature at 2 m, daily mean"),
    ("tmax", "°C", "POWER T2M_MAX"),
    ("tmin", "°C", "POWER T2M_MIN"),
    ("rain", "mm/day", "POWER PRECTOTCORR, bias-corrected precipitation"),
    ("rh", "%", "POWER RH2M"),
    ("ws", "m/s", "POWER WS2M, wind speed at 2 m"),
    ("rs", "MJ/m²/day", "POWER ALLSKY_SFC_SW_DWN, all-sky shortwave"),
    ("tdew", "°C", "POWER T2MDEW, dew/frost point"),
    ("ps", "kPa", "POWER PS, surface pressure"),
    ("et0", "mm/day", "FAO-56 Penman-Monteith reference ET0 computed by Field Shift"),
    ("normal_t2m", "°C", "Day-of-year normal 1985-2024 (±7-day window mean)"),
    ("normal_tmax", "°C", "Day-of-year normal 1985-2024 (±7-day window mean)"),
    ("normal_tmin", "°C", "Day-of-year normal 1985-2024 (±7-day window mean)"),
    ("normal_rain", "mm/day", "Day-of-year normal 1985-2024 (±7-day window mean)"),
    ("normal_et0", "mm/day", "Day-of-year normal 1985-2024 (±7-day window mean)"),
];

/// Daily rows `from..=to` (defaults: the 365 days ending at the last complete day). Missing values are null.
pub fn build_series(d: &PowerDaily, from: Option<NaiveDate>, to: Option<NaiveDate>) -> anyhow::Result<SeriesResponse> {
    let all = days::days(d);
    let end = end_of(&all)?;
    let last = all.last().map(|x| x.date).unwrap_or(end);
    let to = to.unwrap_or(end).min(last);
    let from = from.unwrap_or(to - Duration::days(364)).max(d.start);
    anyhow::ensure!(from <= to, "from must be on or before to (data {}..{})", d.start, last);
    let doy = normals::doy_normals(&all);
    let rows = days::slice(&all, from, to).iter().map(|x| {
        let n = &doy[normals::doy_index(x.date)];
        SeriesRow { date: x.date, t2m: x.t2m, tmax: x.tmax, tmin: x.tmin, rain: x.rain, rh: x.rh, ws: x.ws, rs: x.rs,
                    tdew: x.tdew, ps: x.ps, et0: x.et0, normal_t2m: n.t2m, normal_tmax: n.tmax, normal_tmin: n.tmin,
                    normal_rain: n.rain, normal_et0: n.et0 }
    }).collect();
    let period = format!("{from}..{to}");
    let columns = SERIES_COLUMNS.iter().map(|(k, u, m)| {
        let method = if *k == "et0" { format!("{m}: {}", et0::METHOD) } else { m.to_string() };
        let p = if k.starts_with("normal_") { normals::WINDOW_LABEL.to_string() } else { period.clone() };
        Column { key: k, unit: u, meta: Meta::new(p, method) }
    }).collect();
    Ok(SeriesResponse { location: location(d), from, to, data_through: end, columns, rows, attribution: attribution(d) })
}

/// CSV text of a series response (header row = column keys).
pub fn series_csv(s: &SeriesResponse) -> String {
    let f = |x: Option<f64>| x.map(|v| v.to_string()).unwrap_or_default();
    let mut out = SERIES_COLUMNS.iter().map(|c| c.0).collect::<Vec<_>>().join(",");
    out.push('\n');
    for r in &s.rows {
        let cells = [f(r.t2m), f(r.tmax), f(r.tmin), f(r.rain), f(r.rh), f(r.ws), f(r.rs), f(r.tdew), f(r.ps), f(r.et0),
                     f(r.normal_t2m), f(r.normal_tmax), f(r.normal_tmin), f(r.normal_rain), f(r.normal_et0)];
        out.push_str(&format!("{},{}\n", r.date, cells.join(",")));
    }
    out
}

#[derive(Clone, Debug, Serialize)]
pub struct MonthlyResponse {
    pub location: Location,
    pub data_through: NaiveDate,
    pub normals_window: NormalsWindow,
    /// Last 12 calendar months ending with the month of `data_through`; each month carries every indicator with
    /// its value and its normal band (mean, sd, p10, p90) over the same dates. The last month is usually partial.
    pub months: Vec<MonthPoint>,
    pub attribution: Attribution,
}

#[derive(Clone, Debug, Serialize)]
pub struct MonthPoint {
    pub month: String,
    pub partial: bool,
    #[serde(flatten)]
    pub period: PeriodSummary,
}

pub fn build_monthly(d: &PowerDaily, opts: &Options) -> anyhow::Result<MonthlyResponse> {
    let all = days::days(d);
    let end = end_of(&all)?;
    let (p95, _) = normals::rain_p95(&all);
    let mut first = NaiveDate::from_ymd_opt(end.year(), end.month(), 1).unwrap();
    for _ in 0..11 { first = NaiveDate::from_ymd_opt(first.pred_opt().unwrap().year(), first.pred_opt().unwrap().month(), 1).unwrap(); }
    let mut months = vec![];
    let mut m = first;
    while m <= end {
        let last = normals::last_of_month(m);
        let to = last.min(end);
        months.push(MonthPoint {
            month: m.format("%Y-%m").to_string(),
            partial: to < last,
            period: indicators::period(&all, "month", m, to, opts, p95),
        });
        m = last.succ_opt().unwrap();
    }
    Ok(MonthlyResponse { location: location(d), data_through: end, normals_window: normals_window(), months, attribution: attribution(d) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// PowerDaily with a plain annual cycle from 1981 to 2026-11-09 (all inputs present).
    fn fake() -> PowerDaily {
        let start = NaiveDate::from_ymd_opt(1981, 1, 1).unwrap();
        let n = (NaiveDate::from_ymd_opt(2026, 11, 9).unwrap() - start).num_days() as usize + 1;
        let t: Vec<Option<f64>> = (0..n).map(|i| Some(20.0 + 4.0 * (i as f64 * 2.0 * std::f64::consts::PI / 365.25).cos())).collect();
        let shift = |k: f64| t.iter().map(|x| x.map(|v| v + k)).collect::<Vec<_>>();
        let c = |v: f64| vec![Some(v); n];
        PowerDaily { lat: -11.47, lon: -77.21, elevation_m: 224.43, start, sources: vec!["MERRA2".into()], api_version: "test".into(),
            fetched_at: Utc::now(), t2m: t.clone(), t2m_max: shift(4.0), t2m_min: shift(-4.0), prectotcorr: c(0.5), rh2m: c(80.0),
            ws2m: c(3.0), allsky_sfc_sw_dwn: c(20.0), t2mdew: shift(-3.0), ps: c(98.9) }
    }

    #[test]
    fn summary_series_monthly_build() {
        let d = fake();
        let s = build_summary(&d, &Options::default()).unwrap();
        assert_eq!(s.data_through, NaiveDate::from_ymd_opt(2026, 11, 9).unwrap());
        assert_eq!(s.periods.len(), 3);
        assert_eq!(s.periods[2].from, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        assert_eq!(s.pests.len(), 3);
        assert_eq!(s.monthly_normals.len(), 12);
        let ser = build_series(&d, None, None).unwrap();
        assert_eq!(ser.rows.len(), 365);
        assert!(ser.rows[0].et0.unwrap() > 1.0);
        assert!(series_csv(&ser).lines().count() == 366);
        let m = build_monthly(&d, &Options::default()).unwrap();
        assert_eq!(m.months.len(), 12);
        assert_eq!(m.months[0].month, "2025-12");
        assert!(!m.months[10].partial && m.months[11].partial);
    }
}
