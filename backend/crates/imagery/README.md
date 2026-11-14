# imagery

Sentinel-2 / Landsat NDVI, true-colour snapshot and NASADEM slope via Microsoft Planetary Computer (no account). Routes under `/api/imagery`: `ndvi`, `scenes`, `terrain` (GET query or POST JSON), `snapshot.png`, `snapshot.json`.

| File | What it does |
|------|--------------|
| `src/lib.rs` | Routes, request parsing, public API (`ndvi_series`, `scenes`, `terrain`, `snapshot`), DB cache wrapper, live tests |
| `src/pc.rs` | Planetary Computer client: STAC search with paging, data-API statistics, retries, field geometry and cache keys |
| `src/stac.rs` | Scene search for Sentinel-2 L2A and Landsat 8/9 C2 L2, credits and licences |
| `src/ndvi.rs` | NDVI math and band-math, per-field cloud screening, series merge, bare-soil and crop-cycle detection |
| `src/terrain.rs` | NASADEM grid (.npy) reader, Horn slope, elevation and slope over the field |
| `src/snapshot.rs` | True-colour PNG of the latest clear Sentinel-2 scene, disk cache in `backend/cache/imagery/` |
