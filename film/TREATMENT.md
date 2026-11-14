# Field Shift: demo film treatment

Two films, same picture, ≤ 30 s, 1920×1080, 30 fps: **English** (submitted) and **Spanish** (for the owner). Narration: Gemini TTS, voice Algenib (`tools/gemini_tts.py`), checked with faster-whisper. Styles fused (owner's rule): Satellite Briefing grammar for the opening, Mascot Screencast grammar for the app, with no mascot: the cursor is the farmer, the lock-on reticle is the software.

## Three candidate structures

1. **One continuous dive.** Orbit at sunrise → dive → lock-on on the field → the reticle opens into the app window → farmer sends a photo → readings land → pre-diagnosis → climb back to orbit. A single journey down and up; the bookend is the planet.
2. **Before / after on one field.** Locked nadir frame; the farmer's question arrives as a caption; time scans across Sentinel-2 dates (green, bare, green), then the app answers. Strong evidence moment, but 30 s is too short to also show the app.
3. **The reading that changed the plan.** Open on the amber number (+5 °C), pull back to the field, then to the app explaining what it means for aphids and rotation. Hook is strong, but it starts with an abstract number, and judges need to see the product.

**Chosen: 1, the continuous dive**, because it shows in 30 s what the product is (NASA data → this exact field → advice in the farmer's hands) and its native move (the reticle that locks onto the field and then becomes the app window) is the thread of both styles. The ending echoes the opening: the climb back to orbit.

## Logline and arc

A farmer in the Huaral valley asks what is happening to the crop; the software dives from orbit to that one field, checks NASA data and the farmer's photos, and hands back a pre-diagnosis the agronomist can verify.
Setup (orbit, the question) → turn (lock-on: this field, this season is much warmer than its 40-year normal, pests develop faster) → ending (options and a technical sheet; back to orbit).

## Benchmark

- Learn: the scale-as-camera continuity of a single powers-of-ten zoom; calm guided-tour product films (one feature per sentence, real UI, cursor smoothing).
- Don't take: any layout, typeface, music, shot or logo of those works. No NASA logos or insignia.

## Shot list (≈ 30 s)

| # | Time | Framing / move | What | Why |
|---|------|----------------|------|-----|
| 1 | 0.0–4.0 | Oblique orbit, limb in frame, slow creep | Sunrise over the Pacific coast of South America; searching reticle over Peru; "ALT 420 km", "NASA Blue Marble" | Scale and context; hook is the planet at dawn |
| 2 | 4.0–8.5 | Dive, log zoom, tilt to nadir | Plates swap Blue Marble → Sentinel-2; reticle snaps onto the field with overshoot; outline prints; "Huaral valley, Peru" | Going to look for yourself |
| 3 | 8.5–10.5 | Reticle opens into the app window on a dawn wallpaper | App workspace over the field; chapter pill "01 Your field" | The software's agent becomes the frame (native move) |
| 4 | 10.5–15.5 | Push-in on chat | Cursor drops two photos; reticle snaps onto one tagged "blurry"; consultant asks for a closer shot | Cursor = farmer, reticle = software |
| 5 | 15.5–21.0 | Rack focus to the readings column, ▶▶ tag | Readings land, each with source and date; the temperature anomaly lands in amber | One finding per screen |
| 6 | 21.0–25.5 | Push-in + vignette | "Pre-diagnosis · not a diagnosis"; rotation strategies; "Export technical sheet" | The one promise: confirm with your agronomist |
| 7 | 25.5–30.0 | Climb back to orbit | Valley shrinks under the reticle; end card "Field Shift · team InnoHuando · NASA Space Apps Challenge 2026"; "SI-assisted visuals" mark | Ending echoes the opening |

## Narration (draft, to time against picture)

EN: "This is the Huaral valley, from orbit. Field Shift dives to one farmer's field. The farmer sends photos; NASA data checks them. This season runs five degrees warmer than its forty-year normal, so aphids breed faster. Three options, every number cited. A pre-diagnosis your agronomist can verify."

ES: "Este es el valle de Huaral, desde la órbita. Field Shift baja hasta el campo de un agricultor. El agricultor envía fotos; los datos de la NASA las contrastan. Esta campaña está cinco grados más cálida que su normal de cuarenta años: los pulgones se multiplican más rápido. Tres opciones, cada número con su fuente. Un prediagnóstico que tu agrónomo puede verificar."

"Field Shift" is said in English in both films.

## Sound

Dry telemetry palette for orbit and dive (lock-on ping, ticks, low pulse, riser for the dive); the app section switches to a soft hi-fi bed (piano, brushed kit) with UI foley (clicks, a shutter for photos, pops for readings). Suck-out before the amber reading; the lock-on ping returns at the end. Voice ~10 dB over music, −14 LUFS.

## Titles and subtitles

Transcript strip bottom-left in the briefing part (IBM Plex Mono instruments, Source Sans 3 text); frosted caption pills in the app part. Subtitles burned in and exported as `.srt` in the film's language. Credits and the "SI-assisted visuals" mark on screen; AI narration acknowledged in the description and file metadata.

## Material

Real imagery only: NASA Blue Marble, VIIRS night lights, Sentinel-2 L2A (Contains modified Copernicus Sentinel data 2026). The app is the real app (recorded via headless browser), not a redraw.
