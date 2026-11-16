# Field Shift: tutorial film treatment

**What:** the long film, about 2:30, embedded on fieldshift.earth ("Watch how it works") and posted on YouTube. English first (this cut); Spanish later from the same picture with a second narration.
**Styles fused (owner's rule, as in the 30 s film):** Satellite Briefing for the opening, the dive and the readings (real imagery, thin measurement lines, one reading at a time, every number sourced); Mascot Screencast for the walkthrough, with no mascot: the cursor is the farmer, the lock-on reticle is the software.
**Frame:** 1920×1080, 30 fps (same as the 30 s film), −14 LUFS, captions burned in and exported as `.srt`.

## Three candidate structures

1. **One farmer's season, chapter by chapter.** Problem over the planet → dive to the pea field in Huaral → five chapters that follow the app's own order (field, conversation, NASA data, plan, trust) → climb back to orbit. Clear for a tutorial; risk: a flat list.
2. **Question and answer.** Open on the farmer's question typed in the chat ("Why are my peas full of aphids?") and let every chapter be the software answering one part of it. Strong hook, but it hides the field picker and the intake, which a tutorial must teach.
3. **Scale ladder.** Start at the planet, then step down through the resolutions (POWER ~50 km → SMAP 9 km → Sentinel-2 10 m → the leaf in the photo), and explain the app on the way. Elegant for the data, weak as a how-to.

**Chosen: 1**, with the turn of 2 placed inside it: the readings chapter is the turn (the season is much warmer, aphids breed faster), and the scale idea of 3 survives as the instrument labels the reticle prints on each reading (dataset · resolution · date). The judges' criteria map onto the chapters: Impact (opening), Relevance (NASA data), Validity and usability (the walkthrough and the trust chapter), Creativity (degree-day pests, organic allies, "What if I grow…?"), Presentation (the whole).

## Logline and arc

A farmer in the Huaral valley wants to know what to do with a pea field in a season that feels too hot; Field Shift dives from orbit to that field, listens, reads NASA data and hands back a cited plan the agronomist can confirm.
Setup (pests take up to 40 % of the world's crops; warmth speeds them up) → turn (this field: +5 °C, three more aphid generations) → ending (a plan for pests, soil and rotation, a technical sheet, back to orbit).

## Benchmark

- Learn: calm guided-tour product films (one feature per sentence, real UI, smoothed cursor, the camera pushing to what the voice names); powers-of-ten continuity for the dive.
- Don't take: any layout, typeface, melody, shot or logo of those works. No NASA logos or insignia.

## Shot list and beat sheet (as planned; final times in `src/timeline.json`)

| # | Time | Framing / move | What | Why |
|---|------|----------------|------|-----|
| 1a | 0–1 | Oblique orbit at dawn over Peru | Title "Field Shift" | Hook: the planet at sunrise |
| 1b | 1–5 | Four cuts with whip blur: the globe locks on the US Midwest, Punjab, the Rift Valley, the Pampas | Line 1; the reading "Crops lost to pests · up to 40%" counts up | The problem is planetary |
| 1c | 5–9 | Back over Huaral; slow push | Line 2; NASA POWER monthly trace over its 40-year band; aphid generations fill two rows (season faster) | The cause, in real NASA data |
| 1d | 9–13 | Same, slow push | Line 3; field log (illustrative): sprays fill in, costs climb, soil life fades | The consequence, from the interview |
| 2' | 12–20 | Dive | Footprints lock on at true size: POWER ~50 km → SMAP 9 km → Sentinel-2 10 m grid | Native move: the scale ladder (which data describes the region, which the field) |
| 2 | 13–23 | Dive, log zoom to nadir; lock-on on the field | Lines 4–5; "Huaral valley, Peru" | Going to look for yourself |
| 3 | 23–40 | Reticle opens into the app window; field picker; push to the example card, then to the outline and the area | Ch. 01 Your field: lines 6–7; cursor clicks "Example site", "Draw outline", "Use this field" | Native move: the software's reticle becomes the frame |
| 4 | 40–62 | Whip pan; intake form, push to "I don't know"; whip to the chat, push to the photos and the camera button | Ch. 02 Tell it: lines 8–10 | The farmer speaks; the software asks for what's missing |
| 5 | 62–94 | Whip pan; dashboard wide, then the reticle hops tile by tile with an instrument label; rack to the 40-year chart; push to the aphid tile | Ch. 03 NASA reads your field: lines 11–15; the amber reading lands after a near-silence | The turn |
| 6 | 94–124 | Whip pans between Prevention, Organic and Rotation; slow pushes on the cards; the cursor types "avocado" | Ch. 04 A plan to act: lines 16–20 | The answer is options and allies, not a verdict |
| 7 | 124–140 | Rotation's pre-diagnosis card with a vignette; cursor to Download | Ch. 05 Why trust it: lines 21–22 | The one promise |
| 8 | 140–152 | Window closes back into the reticle; climb to orbit; end card with fieldshift.earth | Line 23; credits; "SI-assisted visuals" | Ending echoes the opening |

## Narration (English)

1. Every year, pests destroy up to forty percent of the world's crops.
2. And as the climate warms, many of them breed faster.
3. So farmers spray more often, pay more, and their soil grows poorer.
4. Field Shift turns NASA's view of the Earth into advice for one field.
5. Here is how it works, on a pea field in the Huaral valley, in Peru.
6. Start with your field. Use your GPS, search a place, or open the example site.
7. Drop a point, or draw the outline. Field Shift measures the area for you.
8. Then tell it what you would tell your agronomist.
9. What grows now, what grew before, how you water. If you don't know, say so.
10. Send photos at full resolution, or simply talk. It asks for what is missing, like a closer look at the underside of a leaf.
11. Meanwhile, NASA data reads your field.
12. Forty years of NASA POWER weather show these weeks running five degrees warmer than normal.
13. NASA's SMAP satellite measures soil moisture. Sentinel-2 maps crop vigor, ten meters per pixel.
14. Every reading shows its source, its date, and whether it describes your field or the whole region.
15. From the temperature, degree-days count pest generations: about three more pea aphid generations than normal.
16. Then the readings become a plan.
17. Prevention ranks the pests likely on your field, with NASA's GEOS-FP weather forecast.
18. Organic care names the natural enemies to protect, like lady beetles and parasitoid wasps, and beneficial fungi such as Trichoderma.
19. Rotation compares your current rotation with two alternatives, for soil health, water and pest cycles.
20. And you can ask: what if I grow avocado? It is scored the same way.
21. It all ends in a pre-diagnosis, not a diagnosis.
22. Download the technical sheet, with every number cited, and confirm it with your agronomist.
23. NASA watches the whole planet. Field Shift helps you read your own field.

Every number and claim: `FACTS.md`.

## Cue map

- Orbit and dive (0–23 s): no beat grid; the telemetry palette of the 30 s film (FM pings, ticks that divide as the camera falls, pulse bass, detuned pad, riser), timpani and the lock-on ping at the lock-on.
- Walkthrough (23–140 s): soft hi-fi bed at 92 BPM (upright piano, pizzicato bass, shaker, soft hats), four-bar phrases; each chapter seam (whip pan) lands on a downbeat with a low tom and a whoosh; reticle snaps get small FM blips; clicks, typing and photo pops as foley.
- Near-silence before the amber reading (line 12): drums and piano drop for one bar, the reading lands on glockenspiel and vibraphone.
- Trust (124–140 s): drums out, bowed vibraphone pad under the promise.
- Climb and end (140–152 s): the lock-on ping returns as the window closes; ticks slow down; timpani and a bowed D chord under the end card.

## Sound design

| Section | Ambience | Foley | Music |
|---|---|---|---|
| Orbit, dive | high thin air noise | pings, ticks, riser, lock-on ping | pulse bass, pad, timpani |
| Walkthrough | soft room tone | clicks, keyboard taps, shutter and pops for photos, whooshes, low tom on seams, FM blips on snaps | piano, pizz bass, shaker, hats |
| Readings | room tone | blips per tile | bar of near-silence, then glock and vibes |
| Trust | room tone | one click on Download | bowed vibraphone pad |
| Climb, end | air noise | lock-on ping, slowing ticks | timpani, bowed D chord |

## Titles and captions

As in the 30 s film: transcript strip bottom-left over the globe (IBM Plex Mono instruments, Source Sans 3 text), frosted caption pills bottom-centre in the app, chapter pills top-left (01–05), instrument labels in Plex next to the reticle in chapter 03, end card in Source Serif 4 italic. "SI-assisted visuals" on every frame (the chat shows the app's AI-assisted example photos). Captions are the narration lines, split where a line runs long.

## Material

Real imagery only: the product's globe (NASA Blue Marble, VIIRS via GIBS; Sentinel-2 L2A plates), and screenshots of the live app at fieldshift.earth (bundled example analysis `demo-huaral`, labelled "Demo data" in the app). Narration: Gemini TTS, voice Algenib (AI-generated audio, acknowledged on screen and in the file metadata).
