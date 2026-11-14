# climate

NEX-GDDP-CMIP6 projection (2031-2050 vs 1995-2014, SSP2-4.5 and SSP5-8.5) served from precomputed example-site files. Mounted at `/api/climate`: `GET /projection?lat=&lon=`.

| File | What it does |
|------|--------------|
| `src/lib.rs` | Router and public fn `projection` |
| `src/projection.rs` | Reads `backend/data/projection_*.json`, nearest site within 50 km (flagged), gap elsewhere |
