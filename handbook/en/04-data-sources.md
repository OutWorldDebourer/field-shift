# 4. NASA and satellite data in the product

Every reading in Field Shift carries its source, date, resolution, licence and a scale badge. **Field-level** readings measure the drawn plot; **regional context** readings describe the surrounding area and are never presented as a measurement of the plot.

| Reading | Source and dataset | Access | Resolution · badge | What the farmer sees |
|---------|--------------------|--------|--------------------|----------------------|
| Temperature vs 40-year normal, rain, reference evapotranspiration (FAO-56), water balance, heat-stress days, growing degree-days | NASA POWER daily API, AG community (MERRA-2, CERES SYN1deg; latest weeks from GEOS-IT / FLASHFlux near-real-time streams, labelled provisional) | No account | ~0.5° × 0.625° (~50 km) · regional | "Much warmer than normal (+5.0 °C)", "irrigation must cover this deficit" |
| Pest generations (degree-days) | NASA POWER temperature + published thresholds (crop's own pests from the crop knowledge base; reference pests: *Myzus persicae*, *Bemisia tabaci*, *Spodoptera frugiperda*) | No account | ~50 km · regional | "Pea aphid: 13.3 vs 10.3 generations this season" |
| Crop vigor (NDVI) series, bare-soil periods, crop cycles | Sentinel-2 L2A (10 m) and Landsat 8/9 Collection 2 L2 (30 m), cloud-masked mean over the outline | Microsoft Planetary Computer, no account | 10–30 m · field-level | "High vigor (0.78)", bare soil Nov–Dec 2025 |
| True-colour picture of the field | Sentinel-2 L2A `visual` asset, newest clear scene | Planetary Computer | 10 m · field-level | The field under the glass interface |
| Slope and elevation | NASADEM (SRTM reprocessed), Horn slope over the outline | Planetary Computer | 30 m · field-level | "Nearly flat (~3 %)" |
| Soil moisture (top 5 cm, root zone) vs the same date in past years | SMAP L4 SPL4SMGP v8 (NSIDC), cloud OPeNDAP | Earthdata Login | 9 km · regional | "Close to the same date in past years" |
| Satellite rain, last 30 days | GPM IMERG Late daily v07 (GES DISC), OPeNDAP | Earthdata Login | 0.1° (~10 km) · regional | "2 rainy days, 4.4 mm" |
| Long-term water storage | GRACE/GRACE-FO JPL mascon CRI RL06.3 v4 (PO.DAAC), pre-extracted per example site | Earthdata Login | ~300 km · regional | "Slowly wetting since 2002" |
| Climate projection ~2040 | NEX-GDDP-CMIP6 (NASA), openly licensed models only | Planetary Computer | 0.25° · regional | "Projection, not a forecast" (shown only where computed) |
| Globe | NASA Blue Marble Next Generation, VIIRS Black Marble, VIIRS true colour (GIBS); Sentinel-2 plates | GIBS, Planetary Computer | — | The dive from orbit to the field |

## Honesty rules in code

- A value without a source is an information gap, never a guess (crop knowledge base and evidence package).
- The consultant may only cite numbers from the evidence package; every number is shown with source and date.
- Recent POWER days are labelled provisional; a cross-check against ERA5 confirmed the warm 2026 (see `research/01_api_tests.md`).
- Pest generations are computed only where a published degree-day threshold exists; otherwise the guidance is qualitative.
