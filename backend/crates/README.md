# crates

| Crate | What it does |
|-------|--------------|
| `api` | Binary `fieldshift`: config, DB pool, migrations, CORS, mounts every router under `/api` |
| `common` | Config (env.txt, infra/.env; secrets redacted), AppState, ApiError, source cache, field geometry |
| `power` | NASA POWER daily series, 40-year normals and agro indicators |
| `imagery` | Sentinel-2 / Landsat NDVI and NASADEM slope via Planetary Computer |
| `earthdata` | SMAP, IMERG and GRACE through Earthdata Login |
| `climate` | NEX-GDDP-CMIP6 projection |
| `crops` | Open crop knowledge: seed base + AI-researched trait cards |
| `analyses` | Analyses, fields, messages, photos, sections storage |
| `consultant` | Claude consultant and scripted demo mode |
| `export` | Technical sheet, CSV, share link |
| `forecast` | NASA GEOS-FP 10-day forecast |
