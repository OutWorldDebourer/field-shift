# Credits: Field Shift demo films

Applies to `field-shift-demo-en.mp4`, `field-shift-demo-es.mp4` and `loop/hero-loop.*`.
Team InnoHuando, NASA Space Apps Challenge 2026. No NASA logos or insignia are used; NASA data is credited as a source only.

## Picture

| Asset | Source | Licence / terms |
|---|---|---|
| Globe, orbit to field (0–9.6 s, 25.2–29.9 s, hero loop) | Frames rendered from the product's own globe (`frontend/globe.html`, `frontend/src/globe/`) with headless Chromium | Team's code (Apache-2.0); imagery below |
| NASA Blue Marble: Next Generation shaded relief | NASA Global Imagery Browse Services (GIBS), layer `BlueMarble_ShadedRelief` | Public domain (NASA); credited "NASA Blue Marble" on screen |
| VIIRS Black Marble night lights; VIIRS SNPP corrected reflectance (cloud veil, 2026-10-06) | NASA GIBS | Public domain (NASA) |
| Landsat WELD annual composite (small patch over Lima in the Blue Marble plates) | NASA/USGS via GIBS | Public domain |
| Sentinel-2 L2A plates of the Huaral valley (2026-10-06) and the app's field imagery (2026-11-13) | Copernicus Sentinel-2 via Microsoft Planetary Computer | Free, full and open; "Contains modified Copernicus Sentinel data 2026" (on screen) |
| EOX Sentinel-2 cloudless 2016 (map base outside the field, in the app's map) | EOX IT Services | CC BY 4.0 (2016 edition only); credited in the app's map attribution |
| App screens (field selection, chat, dashboard, rotation) | Screenshots of the live Field Shift app (fieldshift.earth), bundled example analysis `demo-huaral`, re-captured 2026-11-15 (`src/capture-app.mjs`) | Team's work |
| Pea photos in the chat (`frontend/public/img/pea-*.jpg`) | Crops of AI-assisted design frames made by the team | AI-assisted visuals: the film carries the "SI-assisted visuals" mark on every frame |
| Leaflet map library (inside the app screens) | leafletjs.com | BSD-2-Clause |
| Fonts: IBM Plex Mono, Source Sans 3, Source Serif 4 | Google Fonts / Adobe, self-hosted from `frontend/public/fonts/` | SIL Open Font License 1.1 |
| Cursor, lock-on reticle, window frame, chapter and caption pills, end card | Drawn in code for this film (`film/src/comp.html`) | Team's work |

The farmer's cursor and the software's reticle are overlays; everything inside the app window is the real app as rendered by the browser.

## Sound

| Asset | Source | Licence |
|---|---|---|
| Narration, English and Spanish | AI voice: Google Gemini TTS, voice "Algenib" (`tools/gemini_tts.py`), checked with faster-whisper | AI-generated audio, acknowledged on screen and in the file metadata ("Narration: AI voice, Gemini TTS") |
| Score (upright piano, pizzicato and bowed double bass, shaker, hi-hat, glockenspiel, vibraphone, timpani, horn, toms) | Original composition for this film, played from sample libraries: VS Chamber Orchestra: Community Edition and Versilian Community Sample Library (Versilian Studios) | CC0 1.0 |
| Telemetry pings, ticks, pulse bass, pads, riser, UI clicks, pops, shutter, whooshes | Synthesised in code (`film/src/mix.py`) | Team's work |

## Tools

Rendering, sampler and mastering tools from the Lemo-Opuscar film library (MIT licence; playwright-core, ffmpeg, numpy/scipy, soxr). Speech check: faster-whisper (MIT).
