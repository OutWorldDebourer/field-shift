# Facts: every claim and number in the demo films

Sources: the app's bundled example analysis `demo-huaral` on fieldshift.earth (screens re-captured 2026-11-15, labelled "Demo data" in the app) and the globe's camera. Times are film seconds.

| Time | On screen / spoken | Value | Source and method |
|---|---|---|---|
| 0–9 | "Huaral valley, Peru · 11.47°S 77.21°W" (globe label) | field centre −11.4729, −77.2061 | Field ring in the analysis (`evidence.field.center`); same point as the app's example site |
| 0–8 | "ALT 420 km" … "ALT 2.2 km", scale bar | camera altitude | The globe's camera (`frontend/src/globe/dive.ts`: orbit at 420 km, nadir end at 2.2 km); scale bar from the same camera |
| 1.3–7.5 | Footprints during the dive: "NASA POWER · ~50 km cell", "NASA SMAP · 9 km", "Sentinel-2 · 10 m pixels" | 0.5° × 0.625° POWER cell (≈ 55.6 × 68.2 km at 11.5° S); 9 km SMAP L4 grid; 10 m Sentinel-2 pixels | Drawn at true ground size from the globe camera's scale on each frame (`work/globe-dive/cams.json`) |
| 0–9 | Imagery credits "NASA Blue Marble", "Contains modified Copernicus Sentinel data 2026" | | `frontend/public/globe/CREDITS.md` |
| 8.5–10.3 | "5.1 ha" (field selection) | 5.08 ha | `evidence.field.area_ha`, polygon area of the drawn field |
| 10.4–13 | "The farmer sends photos and NASA data checks them"; photo 3 tagged "blurry"; "+5.0 °C", "0.06 m³/m³", "0.78" in the readings panel | | The app's bundled example conversation (`frontend/src/shared/demo/huaral.ts`, labelled "Demo data") over the same evidence numbers as the analysis below. The photo-quality tag is the example's scripted tag; the photos are AI-assisted design crops (see CREDITS.md) |
| 14–17.6 | "These weeks run five degrees warmer than the forty-year normal"; tile "+5.0 °C vs normal" tagged "provisional", "12 Oct–10 Nov 2026 · vs 1985–2024 · ~50 km · regional context" | +5.0 °C | NASA POWER Daily API v2 (community AG, T2M), https://power.larc.nasa.gov/api/temporal/daily/point. Method as stored: "22.8 °C mean vs 1985–2024 normal 17.8 °C over the same dates (1985 to 2024). Latest weeks come from POWER near-real-time streams and are provisional." ~0.5° × 0.625° grid cell: regional context, not a field measurement. 1985–2024 = 40 years |
| 16–17.5 | Chart "This season vs 40-year normal" | monthly T2M vs normal band | `evidence.climate_monthly` (NASA POWER), normal 1985–2024 |
| 17.6–19.6 | "so aphids breed faster"; tile "Pea aphid generations 13.4, about 3.0 more than normal", label "Degree-days · NASA POWER" | 13.4 vs 10.4 generations, 1 Aug–10 Nov 2026 | Pea aphid (*Acyrthosiphon pisum*) degree-day model on NASA POWER temperatures, thresholds as tabulated by UC IPM (https://ipm.ucanr.edu/weather/phenology-models-description/pea-aphid/). "Breed faster" = more generations completed in the same weeks |
| 14–19 | Other dashboard tiles visible: soil moisture 0.06 m³/m³ (NASA SMAP L4, 10 Nov 2026, 9 km), rain 6.7 mm, NDVI 0.78 (Sentinel-2 L2A, 13 Nov 2026, 10 m), water balance −121 mm | | As printed on each tile, with its dataset, date and resolution (`evidence.readings`) |
| 20.45–23.05 | "Three options, every number cited"; "Three rotation strategies, compared" (keep pea; follow with maize; green-manure break, then maize) | 3 strategies | `sections.rotation` of the analysis; each strategy's "Why, evidence, gaps and risks" lists its citations; footer "Every number cites its source." |
| 23.05–25.6 | "A pre-diagnosis your agronomist can verify"; card "PRE-DIAGNOSIS · NOT A DIAGNOSIS", confidence "Medium", "Confirm with your agronomist." | | `prediagnosis` of the analysis, as rendered by the app |
| 26.0–29.96 | "Field Shift · Team InnoHuando · NASA Space Apps Challenge 2026" | | Team and event names |

Spanish film: the same numbers; "Estas semanas están cinco grados más cálidas que la normal de cuarenta años" = the +5.0 °C vs 1985–2024 reading above.
