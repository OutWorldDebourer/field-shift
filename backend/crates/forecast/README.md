# forecast

NASA GEOS-FP weather forecast (GMAO) for a point, mounted at `/api/forecast?lat=&lon=&days=1..10`.

| File | What it does |
|------|--------------|
| `src/lib.rs` | `forecast()` (cached ~6 h per cell and run) and the route |
| `src/geosfp.rs` | Latest 00 UTC run discovery, grid index, 48-hour OPeNDAP blocks read in parallel, local-day aggregation |
