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

## Revision 2026-11-15

Re-cut on the current app (screens re-captured from fieldshift.earth, example `demo-huaral`): the dive now prints the scale ladder (true-size POWER ~50 km cell → SMAP 9 km → Sentinel-2 10 m grid) and the reticle carries instrument labels on the readings (`POWER · ~50 km`, `Degree-days · NASA POWER`). Narration, music grid and shot timing are unchanged.

## Shot list (29.96 s, as built)

| # | Time | Framing / move | What | Why |
|---|------|----------------|------|-----|
| 1 | 0.0–1.1 | Oblique orbit, limb in frame, slow creep | Sunrise over the Pacific coast; the globe's reticle over Peru; "ALT 420 km", "NASA Blue Marble"; title "Field Shift" | Scale and context; hook is the planet at dawn |
| 2 | 1.1–8.3 | Dive (log zoom, tilt to nadir) | Plates swap Blue Marble → Sentinel-2; outline prints; "Huaral valley, Peru · 11.47°S 77.21°W"; at 7.55 the reticle snaps onto the field with overshoot (lock-on ping, timpani) | Going to look for yourself |
| 3 | 8.45–10.35 | The reticle's brackets open out into the app window; the field blurs into a dawn wallpaper | Real field-selection screen (5.1 ha, the app's own reticle on the field); chapter pill "01 Your field"; the cursor clicks "Use this field" | The software's agent becomes the frame (native move) |
| 4 | 10.35–13.55 | Push-in on the chat, then rack focus to the readings | The cursor sends four photos; the reticle snaps onto photo 3 ("blurry"), then onto the temperature reading (+5.0 °C, amber) | Cursor = farmer, reticle = software |
| 5 | 13.55–20.45 | Whip pan (chapter seam), suck-out, push-in tile → chart → tile | Real dashboard, "02 NASA data": "+5.0 °C vs normal" (1985–2024), the chart "This season vs 40-year normal", "14.2 aphid generations, +3.5 vs normal" | One finding per screen |
| 6 | 20.45–23.05 | Whip pan, slow push on the cards | "03 Options": three rotation strategies compared; the reticle moves to "Every number cites its source." | The answer is options, with sources |
| 7 | 23.05–25.6 | Push-in + vignette on the pre-diagnosis card | "04 Pre-diagnosis": "PRE-DIAGNOSIS · NOT A DIAGNOSIS", confidence Medium, the reticle on "Confirm with your agronomist." | The one promise |
| 8 | 25.6–29.96 | The window closes back into the reticle; climb back to orbit | End card "Field Shift / Team InnoHuando / NASA Space Apps Challenge 2026" and credits; "SI-assisted visuals" mark | Ending echoes the opening |

## Narration (recorded, verified with faster-whisper)

EN: "This is the Huaral valley, from orbit. Field Shift dives to one farmer's field. The farmer sends photos and NASA data checks them. These weeks run five degrees warmer than the forty-year normal, so aphids breed faster. Three options, every number cited. A pre-diagnosis your agronomist can verify."

ES (atempo 1.07): "Este es el valle de Huaral, desde la órbita. Field Shift baja hasta el campo de un agricultor. El agricultor envía fotos y los datos de la NASA las contrastan. Estas semanas están cinco grados más cálidas que la normal de cuarenta años. Los pulgones se multiplican más rápido. Tres opciones, cada número con su fuente. Un prediagnóstico que tu agrónomo puede verificar."

"Field Shift" is said in English in both films. Each line is cut from the take at its word timings and placed on its shot (`src/timeline.py`); the Spanish lines are longer, so the picture's beats sit where both languages fit.

## Sound

Dry telemetry palette for orbit and dive (FM pings, ticks that divide to accelerate, pulse bass with 2nd/3rd harmonics, detuned-saw pad, noise riser, timpani and a horn stab on the lock-on); a suck-out before the window opens, then the app section switches to a soft hi-fi bed at 96 BPM (upright piano D–Bm–G–A, pizzicato bass, shaker, soft hi-hat) with UI foley (clicks, a shutter and pops for the photos, whooshes and a low tom on the whip pans, small FM blips when the reticle snaps). Suck-out before the amber reading, which lands on glockenspiel and vibraphone; the drums drop under the promise (bowed vibraphone pad); the lock-on ping returns when the window closes, ticks slow down as the camera climbs, timpani and a bowed D chord under the end card. Voice ~10 dB over music (by RMS, music ducked 4 dB under the voice), −14 LUFS, true peak ≤ −1 dBTP.

## Titles and subtitles

Transcript strip bottom-left in the briefing part (IBM Plex Mono instruments, Source Sans 3 text), above the globe's ALT and scale bar; frosted caption pills at the bottom centre in the app part; chapter pills top-left (01–04, English in both films: same picture). Subtitles burned in and exported as `.srt` in the film's language. Credits and the "SI-assisted visuals" mark on screen; AI narration acknowledged in the description and file metadata.

## Material

Real imagery only: NASA Blue Marble, VIIRS night lights, Sentinel-2 L2A (Contains modified Copernicus Sentinel data 2026). The app is the real app (recorded via headless browser), not a redraw.
