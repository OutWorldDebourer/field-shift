# earthdata

SMAP, IMERG and GRACE at a point through Earthdata Login (bearer token from `env.txt`, never logged). Mounted at `/api/earthdata`: `GET /smap?lat=&lon=[&days=14]`, `/imerg?lat=&lon=[&days=60]`, `/grace?lat=&lon=`.

| File | What it does |
|------|--------------|
| `src/lib.rs` | Router and public fns `smap`, `imerg`, `grace` (cached via `source_cache`) |
| `src/routes.rs` | HTTP handlers |
| `src/meta.rs` | `Provenance`: source, dataset id/version, DOI, resolution, badge, license |
| `src/grid.rs` | EASE-Grid 2.0 9 km and IMERG 0.1° indexing (formula and constants documented) |
| `src/client.rs` | CMR granule search, OPeNDAP `.dap.csv` reads, DAP CSV parser, bounded concurrency |
| `src/ctx.rs` | HTTP client + token + optional cache for the readers |
| `src/smap.rs` | SMAP L4 latest, daily series and same-date comparison with earlier years |
| `src/imerg.rs` | IMERG Late daily rain series and summary |
| `src/grace.rs` | GRACE series and trend from `backend/data/grace_*.json` (example sites), gap elsewhere |
| `tests/network.rs` | Live Huaral reads (`cargo test -p earthdata -- --ignored`) |
