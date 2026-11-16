# Space Apps project page: submission copy

The text entered on https://www.spaceappschallenge.org/2026/find-a-team/innohuando/?tab=project (all fields must be in English). Keep this file in step with the form. Saved on the site 2026-11-15 20:59 (not yet submitted for judging).

## Project Name

Field Shift

## Summary (50–200 words)

Field Shift is a bilingual (English/Spanish) web app that dives from orbit to one farmer's field and turns NASA Earth data into a cited pre-diagnosis for crop rotation and pest care. The farmer picks or draws a field, tells an AI consultant what grows now, what grew before and how the field is watered, and sends photos. Field Shift reads NASA POWER (40 years of weather and degree-days for pest generations), SMAP soil moisture, GPM IMERG rain, GRACE/GRACE-FO water storage, the GEOS-FP forecast, NASADEM slope and NEX-GDDP-CMIP6 projections, together with Sentinel-2 and Landsat crop vigor at 10–30 m. It then compares the farmer's current rotation with two alternatives, ranks the pests likely on that field, names the natural enemies and beneficial fungi to protect, and exports a technical sheet where every number shows its source, date and scale (field-level or regional). It was shaped with an agronomist in the Huaral valley, Peru, where warmer seasons make pests breed faster and sprays more frequent. It supports, and never replaces, the farmer's agronomist.

## Project Demonstration

https://youtu.be/epTY-ji1BXo (30 s, English narration and captions)

## Project

https://fieldshift.earth

## Project Details

See the text block below (entered in the rich-text field).

```
WHAT IT DOES
Field Shift helps a farmer decide what to plant next and how to protect the crop, using NASA data about their own field. It answers the challenge directly: it compares the current rotation with two alternatives, explains why each was suggested, the evidence, the benefits, the information gaps and the risks to check locally, and it always says that the result is a pre-diagnosis to confirm with an agronomist.

HOW IT WORKS
1. Your field. Use your GPS, search a place, open the example site, drop a point or draw the outline; the app measures the area.
2. Tell it what you know. A short form or a guided chat collects the crop now, the crops before, watering, soil and priorities; "I don't know" is a valid answer and becomes a declared data gap. Photos are sent at full resolution, and voice dictation works in English and Spanish.
3. NASA reads your field. A Rust backend builds an evidence package for the exact coordinates: temperature against the 1985–2024 normal, rain, FAO-56 evapotranspiration and water balance, heat-stress days and pest generations from degree-days (NASA POWER), soil moisture (SMAP L4), satellite rain (GPM IMERG), long-term water storage (GRACE/GRACE-FO), a 9-day forecast (GEOS-FP), slope (NASADEM), crop vigor and bare-soil periods (Sentinel-2 at 10 m, Landsat at 30 m) and a 2031–2050 climate projection (NEX-GDDP-CMIP6). Every reading carries its source, date, resolution and a badge: field-level or regional context.
4. A plan to act. Dashboard, Prevention (pest risk per crop with the forecast), Organic care (natural enemies such as lady beetles and parasitoid wasps, beneficial fungi such as Trichoderma, when to act), Rotation (three strategies plus "What if I grow…?" for any crop, scored on soil health, water use and pest-cycle break) and a downloadable technical sheet, CSV data and a share link.

WHY IT MATTERS (IMPACT)
Pests destroy up to 40% of the world's crop production (FAO, 2021), and warming lets many of them complete more generations per season. In the Huaral valley, Peru, the example season runs about 5 °C warmer than its 40-year normal and pea aphids complete about three more generations than normal; farmers answer with more frequent sprays that raise costs and harm soil life. Field Shift turns free NASA data into advice a small farmer can act on, in their language, with integrated pest management and soil care instead of more chemicals. NASA POWER, SMAP, GPM, GRACE and the satellites used are global, so the same steps work for any field.

WHAT IS NEW (CREATIVITY)
- The farmer's field is the unit: the app dives from orbit to one outline and shows, at true size, which data describe that field and which only describe the region.
- Pest pressure is computed, not guessed: degree-day models with published thresholds on NASA temperatures, combined with the GEOS-FP forecast.
- An AI consultant that may only cite numbers from the evidence package, asks for what is missing (for example a closer photo of the underside of a leaf) and marks every gap.
- Rotation options are tested against 2031–2050 climate projections, and any crop can be evaluated on the farmer's own field.

HOW WE KEPT IT VALID
- A value without a source is shown as an information gap, never a guess. Recent NASA POWER days are labelled provisional; regional readings are never presented as field measurements.
- Crop traits, pest thresholds and natural enemies come from a cited knowledge base (FAO-56, FAO Ecocrop, UC IPM, SARE, INIA, SENASA, peer-reviewed papers); AI-researched items are labelled "AI-researched, verify".
- The advice themes (soil care with organic and green manures and Trichoderma, residue management, biological control) come from an interview with agronomist Ing. Cenia, who works with farmers in Huaral.

TOOLS
Rust (axum, tokio, sqlx, reqwest) and MariaDB for the backend and the NASA data services; TypeScript and Vite for the web app; Three.js for the globe; Leaflet and MapLibre for maps; Claude (Anthropic API) for the consultant; Gemini for voice dictation; deployed on a Google Cloud e2-micro VM with Caddy (HTTPS) at fieldshift.earth (domain from the Space Apps global offer). The films were made in code with the Lemo-Opuscar library. Everything is bilingual, English by default with a Spanish switch.

TRY IT
- Finished example (opens in seconds): https://fieldshift.earth/app.html?a=demo-huaral&tab=dashboard
- How-to film (2:21, English): https://youtu.be/siLFiw89jxk
- Cómo funciona (2:21, español): https://youtu.be/MyDycfiTa2w
- 30-second demo in Spanish: https://youtu.be/EF2deFeGko4
```

## Use of Super Intelligence (SI)

Options selected: Research or Information Gathering; Data Analysis and Interpretation; Building SI/ML Into the Project; Coding, Debugging, or Troubleshooting; Writing, Editing, or Translation; Visual Design or Image Creation; Presentation, Video, or Audio. Not selected: Ideation or Brainstorming (the idea is the team's). Our disclosure (`AI_USE.md`, handbook page 8): Claude via Claude Code wrote most of the code and documentation under the team's direction; the in-app consultant is Claude (`claude-opus-5-5`); AI-researched crop items are labelled; illustrative images carry the "SI-assisted visuals" tag; film narration is Gemini TTS, acknowledged on screen and in metadata. The idea and the approach are the team's.

## NASA Data (link text → URL)

| Link text | URL |
|---|---|
| NASA POWER Daily API, Agroclimatology (temperature, rain, radiation, 1985–2024 normal) | https://power.larc.nasa.gov/ |
| SMAP L4 Global 3-hourly 9 km Surface and Root Zone Soil Moisture (SPL4SMGP v8) | https://nsidc.org/data/spl4smgp/versions/8 |
| GPM IMERG Late Precipitation L3 1 day 0.1° v07 (GPM_3IMERGDL) | https://doi.org/10.5067/GPM/IMERGDL/DAY/07 |
| GRACE/GRACE-FO JPL Mascon CRI RL06.3 (terrestrial water storage) | https://podaac.jpl.nasa.gov/GRACE |
| NASA GMAO GEOS-FP forecast | https://gmao.gsfc.nasa.gov/GEOS_systems/ |
| NASADEM elevation (slope) | https://doi.org/10.5067/MEaSUREs/NASADEM/NASADEM_HGT.001 |
| NEX-GDDP-CMIP6 climate projections | https://doi.org/10.7917/OFSG3345 |
| NASA GIBS: Blue Marble, VIIRS Black Marble and VIIRS true colour (globe) | https://nasa-gibs.github.io/gibs-api-docs/ |
| Landsat 8/9 Collection 2 Level-2 (NASA/USGS) | https://landsat.gsfc.nasa.gov/ |

## Space Agency Partner & Other Data (link text → URL)

| Link text | URL |
|---|---|
| ESA Copernicus Sentinel-2 L2A (via Microsoft Planetary Computer) | https://planetarycomputer.microsoft.com/dataset/sentinel-2-l2a |
| Landsat C2 L2 access via Microsoft Planetary Computer | https://planetarycomputer.microsoft.com/dataset/landsat-c2-l2 |
| NASADEM and NEX-GDDP-CMIP6 access via Microsoft Planetary Computer | https://planetarycomputer.microsoft.com/dataset/nasadem |
| EOX Sentinel-2 cloudless 2016 map layer (CC BY 4.0) | https://s2maps.eu/ |
| OpenStreetMap data and Nominatim place search (ODbL) | https://www.openstreetmap.org/copyright |
| OpenFreeMap vector tiles | https://openfreemap.org/ |
| FAO-56 Crop evapotranspiration (Allen et al. 1998) | https://www.fao.org/4/x0490e/x0490e00.htm |
| FAO (2021) Scientific review of the impact of climate change on plant pests | https://openknowledge.fao.org/handle/20.500.14283/cb4769en |
| UC IPM degree-day models (pea aphid thresholds) | https://ipm.ucanr.edu/weather/phenology-models-description/pea-aphid/ |
| Whalon & Smilowitz 1979, green peach aphid model | https://doi.org/10.4039/Ent1111025-9 |
| du Plessis et al. 2020, Spodoptera frugiperda development (CC BY 4.0) | https://doi.org/10.3390/insects11040228 |
| Claude API, Anthropic (consultant) | https://www.anthropic.com/api |
| Google Gemini API (voice dictation, film narration voice) | https://ai.google.dev/ |
| Lemo-Opuscar film library (MIT) | https://github.com/lemomo-ai/lemo-opuscar |
| Three.js (MIT) | https://threejs.org/ |
| Leaflet (BSD-2-Clause) | https://leafletjs.com/ |
| MapLibre GL JS (BSD-3-Clause) | https://maplibre.org/ |
| Fonts: Source Serif 4, Source Sans 3, IBM Plex Mono (OFL), via Fontsource | https://fontsource.org/ |
| Music samples: VS Chamber Orchestra CE and VCSL (CC0) | https://versilian-studios.com/ |
| Field Shift source code (Apache-2.0), with all sources and licences | https://github.com/OutWorldDebourer/field-shift |

The crop knowledge base cites 103 + 41 further sources; they are listed with their licences in the repository (`backend/crates/crops/data/README.md`).
