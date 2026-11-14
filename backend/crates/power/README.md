# power

NASA POWER daily series, 40-year normals (1985-2024) and agro indicators. Routes under `/api/power`: `summary`, `series` (`format=csv` optional), `monthly`.

- `src/lib.rs`: router, cache key, public async `daily` / `summary` / `series` / `monthly`, live `#[ignore]` test.
- `src/client.rs`: POWER daily point request (1981 → today, one call), -999 → missing, column-wise `PowerDaily`.
- `src/days.rs`: row view with derived ET0, slicing, season start, same calendar window in other years.
- `src/et0.rs`: FAO-56 Penman-Monteith ET0 (Allen et al. 1998), tested on FAO-56 Examples 8 and 18.
- `src/gdd.rs`: GDD (averaging) and single-sine degree-days with upper cutoff (Zalom et al. 1983).
- `src/normals.rs`: 1985-2024 normals per calendar window, month and day of year; wet-day rain p95.
- `src/indicators.rs`: period indicators (anomaly, heat/frost days, rain extremes, ET0, water balance, GDD) vs normal.
- `src/waterbalance.rs`: rain − ET0 × Kc (irrigation not modelled).
- `src/pests.rs`: cited pest thresholds and generations this season vs normal.
- `src/summary.rs`: builders for the summary, series (+CSV) and monthly responses.
- `src/meta.rs`: per-value provenance (source, dataset, period, resolution, badge, license, method).
- `src/stats.rs`: mean, sd, percentiles, `NormalStat`.
- `src/routes.rs`: axum handlers and query validation.
