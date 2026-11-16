# Credits: Field Shift tutorial film

Applies to `field-shift-tutorial-en.mp4` and `field-shift-tutorial-es.mp4`. Team InnoHuando, NASA Space Apps Challenge 2026. No NASA logos or insignia are used; NASA data is credited as a source only.

## Picture

| Asset | Source | Licence / terms |
|---|---|---|
| Globe: world tour, orbit, dive and climb (`src/globe-world.mjs`) | Frames rendered from the product's own globe (`frontend/globe.html`, `frontend/src/globe/`) with headless Chromium; climb frames reused from the 30 s film | Team's code (Apache-2.0); imagery below |
| NASA Blue Marble: Next Generation shaded relief; VIIRS night lights and corrected reflectance | NASA Global Imagery Browse Services (GIBS) | Public domain (NASA); credited on screen |
| Sentinel-2 L2A plates and the app's field imagery | Copernicus Sentinel-2 via Microsoft Planetary Computer | Free, full and open; "Contains modified Copernicus Sentinel data 2026" (on screen) |
| EOX Sentinel-2 cloudless 2016 (map base in the app) | EOX IT Services | CC BY 4.0; credited in the app's map attribution |
| App screens (field picker, Load information, chat, dashboard, Prevention, Organic, Rotation, Download) | Screenshots of the live app at fieldshift.earth, example analysis `demo-huaral`, captured 2026-11-15 (`src/capture.mjs`) | Team's work |
| Example pea photos inside the chat | AI-assisted example images shipped with the app (`frontend/public/img/pea-*.jpg`) | AI-assisted visuals: the "SI-assisted visuals" mark is on every frame |
| Leaflet map library (inside the app screens) | leafletjs.com | BSD-2-Clause |
| Fonts: IBM Plex Mono, Source Sans 3, Source Serif 4 | Google Fonts / Adobe, self-hosted from `frontend/public/fonts/` | SIL Open Font License 1.1 |
| Opening panels (FAO reading, NASA POWER trace, generations, field log, footprints), cursor, lock-on reticle, instrument labels, window frame, chapter and caption pills, end card | Drawn in code for this film (`src/comp.html`) | Team's work |

## Sound

| Asset | Source | Licence |
|---|---|---|
| Narration (English and Spanish) | AI voice: Google Gemini TTS, voice "Algenib", model gemini-3.8-flash-tts (`tools/gemini_tts.py`), checked with faster-whisper (`voice/whisper-check-en.txt`) | AI-generated audio, acknowledged on screen and in the file metadata |
| Score (upright piano, pizzicato and bowed double bass, shaker, hi-hat, vibraphone, bowed vibraphone, glockenspiel, timpani, horn, toms) | Original composition for this film, played from sample libraries: VS Chamber Orchestra: Community Edition and Versilian Community Sample Library (Versilian Studios) | CC0 1.0 |
| Telemetry pings, ticks, pulse bass, pads, riser, room tone, UI clicks, pops, shutter, whooshes | Synthesised in code (`src/mix.py`) | Team's work |

## Tools

Rendering, sampler and mastering tools from the Lemo-Opuscar film library (MIT licence; playwright-core, ffmpeg, numpy/scipy, soxr). Speech check: faster-whisper (MIT).
