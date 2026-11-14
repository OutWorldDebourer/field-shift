# API access tests

Run 2026-11-14 (early morning, before the hackathon start) against the pilot point Huaral valley, Peru, lat -11.47, lon -77.21. Raw responses are in `api-samples/`.

## Verified without any account

| Service | Test | Result | What we get |
|---------|------|--------|-------------|
| NASA POWER daily | `https://power.larc.nasa.gov/api/temporal/daily/point?parameters=T2M,T2M_MAX,T2M_MIN,PRECTOTCORR,RH2M,WS2M,ALLSKY_SFC_SW_DWN,GWETROOT,GWETTOP,EVPTRNS&community=AG&longitude=-77.21&latitude=-11.47&start=20251101&end=20251107&format=JSON` | 200, 1.2 s | All 10 parameters. Sources: MERRA-2, SYN1deg. Fill value -999. |
| NASA POWER from 1981 | same, `start=19810101` | 200 | Daily series start in 1981, enough for a 40+ year baseline computed by us. |
| NASA POWER climatology | `.../temporal/climatology/point?parameters=T2M,T2M_MAX,T2M_MIN,PRECTOTCORR,FROST_DAYS&community=AG&...` | 200, 0.7 s | Pre-computed normals, but only for Jan 2001 to Dec 2025 (POWER's message). For a 40-year normal we average the daily series ourselves. |
| Planetary Computer STAC | `GET /api/stac/v1/collections/<id>` | 200 for `sentinel-2-l2a`, `landsat-c2-l2`, `nasadem`, `modis-13Q1-061` (NDVI), `modis-11A2-061` (LST), `modis-16A3GF-061` (ET), `nasa-nex-gddp-cmip6`, `gpm-imerg-hhr`, `era5-pds`, `cop-dem-glo-30`, `esa-worldcover`, `io-lulc-annual-v02` | Search and metadata need no account. |
| Sentinel-2 over the field | `POST /api/stac/v1/search`, point, 2026-09-01..2026-11-13, cloud < 30 % | 3 scenes: 2026-10-09 (0.7 %), 2026-10-06 (4 %), 2026-09-24 (12.7 %), tile T18LTN | Field-level NDVI possible; note no clear scene after Oct 9 (coastal cloud season). |
| Planetary Computer data access | `GET /api/sas/v1/token/sentinel-2-l2a` | 200, anonymous token valid ~24 h | Reading the actual imagery needs only this free anonymous token. |
| NASADEM | STAC search at the point | Tile `NASADEM_HGT_s12w078` | Elevation and slope. |
| NASA GIBS map tiles | WMTS capabilities (5.3 MB) and one IMERG tile for 2026-11-10 | 200, PNG | Ready-made map layers, no account: `IMERG_Precipitation_Rate`, `SMAP_L4_Analyzed_Surface_Soil_Moisture`, `SMAP_L4_Analyzed_Root_Zone_Soil_Moisture`, `MODIS_Terra_L3_NDVI_16Day`, `MODIS_Terra_L3_Land_Surface_Temp_8Day_Day`, `GRACE_Tellus_Liquid_Water_Equivalent_Thickness_Mascon_CRI`, `BlueMarble_NextGeneration`. |
| NASA CMR (granule search) | `https://cmr.earthdata.nasa.gov/search/granules.json?short_name=<id>&point=-77.21,-11.47&sort_key=-start_date&page_size=1` | 200 | Latest granules: SMAP L4 `SPL4SMGP` 2026-11-10 · IMERG Late daily `GPM_3IMERGDL` 2026-11-12 · IMERG Final daily `GPM_3IMERGDF` 2025-09-30 · GRACE/GRACE-FO mascon CRI v4 through 2026-09. |

## Findings that change the design

- **IMERG on Planetary Computer ends in May 2021.** Recent rainfall needs IMERG Late (`GPM_3IMERGDL`, about 1 day behind) from NASA GES DISC, which requires Earthdata Login.
- **SMAP, IMERG and GRACE numbers need Earthdata Login**; their map images (GIBS) do not.
- **POWER's grid cell is coarse** (about 0.5° by 0.625°). Its point elevation at the pilot is 224 m and the cell mixes ocean, valley and hills: label POWER as regional context.
- **NEX-GDDP-CMIP6 shows license "proprietary" in the Planetary Computer catalog.** UNVERIFIED: check NASA's own terms (NCCS) before publishing derived numbers.
- **Coastal clouds**: Sentinel-2 clear scenes are sparse from October on; combine Sentinel-2 and Landsat and show the date of the scene used.

## Verified with the owner's Earthdata Login token

The token lives in `env.txt` under the label `earthdata` (git-ignored) and is sent as `Authorization: Bearer`.

| Service | Test | Result |
|---------|------|--------|
| SMAP L4 (NSIDC, cloud OPeNDAP) | `https://opendap.earthdata.nasa.gov/collections/C3480440870-NSIDC_CPRD/granules/SMAP_L4_SM_gph_20261110T223000_Vv8013_001.h5_u9loH5HD.dap.csv?dap4.ce=/Geophysical_Data/sm_surface[974:975][1102:1103];...` | 200, ~2 s. Cell centred at -11.55, -77.07 (EASE-Grid 2.0, 9 km): surface 0.051 m³/m³, root zone 0.096 m³/m³ on 2026-11-10 22:30 UTC. |
| GRACE/GRACE-FO mascon CRI v4 (PO.DAAC) | Download of the 46 MB file (7.5 s), point extracted with `tools/grace_point.py` | 261 monthly values, 2002-04 to 2026-09, cell centred at -11.25, 282.75 (=-77.25). Range -6.96 to +14.43 cm; linear trend +0.24 cm/year (regional, ~300 km). Output: `api-samples/grace_huaral.json`. |
| IMERG Late daily (GES DISC OPeNDAP) | `https://gpm1.gesdisc.eosdis.nasa.gov/opendap/GPM_L3/GPM_3IMERGDL.07/2026/11/3B-DAY-L.MS.MRG.3IMERG.20261112-S000000-E235959.V07C.nc4.dap.csv?dap4.ce=/precipitation[0:0][1027:1028][785:786];/lon[1027:1028];/lat[785:786]` | 200, 0.7 s, after the owner authorized "NASA GESDISC DATA ARCHIVE" (401 before). 2026-11-12, cell -77.25 / -11.45: 0.195 mm/day; neighbours 0 to 0.24 mm/day (dry season on the desert coast). Variable order is [time][lon][lat]. |

Notes:
- The OPeNDAP granule id must be the full CMR GranuleUR (with its `_u9loH5HD` suffix); a single index `[974]` returns no value, use a range `[974:975]`.
- SMAP at 9 km reads the desert around the valley (0.05 m³/m³ is very dry): it is regional context, not the irrigated field's moisture. Compute the EASE-Grid index from the cell centre, the first attempt landed one cell east.
- GRACE is read with `h5py` from the project's tools environment (`tools/.venv`, git-ignored); the downloaded file is cached in `research/cache/` (git-ignored).

## Cross-check of the 2026 warm anomaly (2026-11-14, during the build)

NASA POWER shows the Huaral field running +4 to +5 °C above its 1985–2024 normal from May 2026 on (Oct 2026: 22.5 °C vs normal 17.7 °C). The newest POWER days come from near-real-time streams (GEOS-IT, FLASHFlux), so we checked an independent reanalysis: ERA5 via the Open-Meteo archive API (research only, not shipped) gives monthly means at the same point of 22.46 °C (Oct 2026) and 22.70 °C (Nov 1–5), against 16.13 °C in Oct 2025. Both sources agree on Oct–Nov; for Jun–Sep 2026 POWER runs about 1.5–2 °C warmer than ERA5. Conclusion: the warm 2026 is real (a strong coastal warm event); the app labels the latest weeks as provisional.

## Correction (2026-11-14, during the build)

The SMAP L4 cell above (row 974, col 1102, centre -11.55, -77.07) is wrong for -11.47, -77.21: checked against the real `cell_lat`/`cell_lon` arrays, the point falls in row 973, col 1100 (centre -11.4783, -77.2562). The backend (`backend/crates/earthdata/src/grid.rs`) uses the corrected EASE-Grid 2.0 formula, covered by unit tests.
