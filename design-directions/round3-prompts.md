# Round 3 prompts: landing + analysis workspace

For the owner to run in the ChatGPT app (image generation). Save the results here as `9-landing-desktop.png`, `10-workspace-desktop.png` and `11-workspace-mobile.png`.

Shared look for all three (already included in each prompt): Daylight Mission Briefing world, white and pale celeste, sunrise warmth, Source Serif 4 + Source Sans 3, hairline rules, no NASA marks.

## 1. Landing page, desktop (landscape 3:2)

```text
Design a desktop landing page (landscape, 3:2, shown flat and full-frame, no browser chrome, no device mockup) for a web app called Field Shift, an entry to the NASA Space Apps Challenge 2026. Field Shift helps small farmers: they send photos of their crop, an AI consultant cross-checks them with NASA Earth observation data, and they get a pre-diagnosis plus crop-rotation and pest-care options to confirm with their agronomist.

Visual world: "daylight mission briefing". Clean, corporate, spacious, calm. White and pale celeste (sky blue) dominate, with the warmth of a sunrise used sparingly (soft peach-amber light on the horizon, amber only for key highlights). Real satellite imagery with thin white measurement lines and camera-reticle corner brackets. Typography: Source Serif 4 regular for large headlines with tight tracking; Source Sans 3 uppercase with wide letter-spacing for small labels; 1px hairline rules; no shadows, no glossy gradients. No NASA logo, no flags, no mission patches, no people.

Layout top to bottom:
1. Thin top bar: wordmark FIELD SHIFT in tracked caps at left; links How it works, NASA data, About; an EN | ES language switch at right.
2. Hero, about 60% of the height: a wide top-down satellite view of an irrigated coastal desert valley (small green and brown fields along a river, dry hills) lit by early sunrise, fading into a white sky at the top. One small field is outlined with a thin white line and white corner brackets, with a tiny label 2.1 ha. On the left, over the white sky, a large serif headline: "Photos from your field. NASA data from above. A pre-diagnosis you can verify." Under it one line of body text: "Field Shift cross-checks your crop photos with satellite and climate data, then suggests rotations, soil care and pest control to review with your agronomist." Two buttons: a solid deep-blue primary button "Start analysis" and an outline button "Try an example site".
3. Below the hero, a three-step strip with thin numbered columns: 01 Capture (send photos and tell us what you grow), 02 Cross-check (temperature, soil moisture, rain, crop vigor, slope, 2040 projection), 03 Pre-diagnosis (soil, pest care, nutrition, rotation, technical sheet).
4. A slim row of six small data labels with tiny line icons: NASA POWER, SMAP, GPM IMERG, Sentinel-2, SRTM, NEX-GDDP.
5. A quiet footer line: "A support tool. It does not replace your agronomist's judgment." and Team InnoHuando · NASA Space Apps Challenge 2026.
```

## 2. Analysis workspace, desktop (landscape 3:2)

```text
Design the main analysis screen of a desktop web app called Field Shift (landscape, 3:2, shown flat and full-frame, no browser chrome, no device mockup). Field Shift helps small farmers: they chat with an AI consultant, send crop photos, and the app cross-checks them with NASA Earth observation data to build a pre-diagnosis.

Visual world: "daylight mission briefing". Clean, corporate, spacious, calm. White and pale celeste dominate, sunrise amber only for highlights and warnings. Typography: Source Serif 4 regular for titles and large figures; Source Sans 3 uppercase with wide letter-spacing for small labels; 1px hairline rules; no heavy shadows. No NASA logo, no flags, no people.

Layout, three zones:
1. Thin top bar: FIELD SHIFT wordmark at left; a field chip "2.1 ha · Huaral, Peru" with a two-way switch My GPS | Example site; an EN | ES switch at right.
2. Left and center (about 60% width): the chat with the consultant. Above the chat, a small top-down satellite thumbnail of the field outlined with a thin white line and corner brackets. Messages: consultant "What are you growing now, and what grew here before?"; farmer "Peas now. Lettuce before."; the farmer sends a row of four photos of green pea plants, photo 3 marked Blurry; consultant "Thanks. Photo 3 is blurry. Take the underside of a leaf, closer. Meanwhile I'm checking NASA data for your field."; a small inline status line with a satellite icon: "Reading NASA POWER, SMAP, Sentinel-2…". At the bottom, a message field "Write or send up to 10 photos" with a camera button.
3. Right column (about 40% width): a vertical stack of six section cards that fill in as the chat goes on, each with an icon, a title, a one-line summary and a state:
   - Your field · NASA data: READY, with three tiny readings: Temperature +1.2 °C vs normal (NASA POWER), Soil moisture drier than normal (SMAP), Crop vigor 0.48 (Sentinel-2).
   - Soil: IN PROGRESS, "Organic matter low. Green manure and Trichoderma-enriched compost suggested."
   - Pest care: IN PROGRESS, "Possible aphids. Beneficial insects: lacewings (Chrysoperla)." with a small amber note "Needs a closer photo".
   - Nutrition: EMPTY, "Needs: last fertilizer used".
   - Rotation: EMPTY, "3 strategies will appear here".
   - Technical sheet: LOCKED, a disabled-looking button "Export PDF" with "Every number cites its source".
   Each card has a thin progress ring; ready cards are white with a celeste check, in-progress cards have a half ring, empty cards are dashed outlines.
```

## 3. Analysis workspace, mobile (portrait 2:3)

```text
Design the main analysis screen of a mobile web app called Field Shift (portrait, 2:3, a single phone screen shown flat and full-frame, no device mockup, no hands). Field Shift helps small farmers: they chat with an AI consultant, send crop photos, and the app cross-checks them with NASA Earth observation data to build a pre-diagnosis.

Visual world: "daylight mission briefing". Clean, corporate, spacious, calm. White and pale celeste dominate, sunrise amber only for highlights. Typography: Source Serif 4 regular for titles; Source Sans 3 uppercase with wide letter-spacing for small labels; 1px hairline rules. No NASA logo, no flags, no people.

Layout top to bottom:
1. Slim top bar: FIELD SHIFT at left, field chip "2.1 ha · Huaral" in the middle, EN | ES at right.
2. A horizontal strip of six round section buttons, each a small icon inside a thin progress ring with a tiny label below: Field (full ring, ready), Soil (half ring), Pests (half ring with a small amber dot), Nutrition (empty ring), Rotation (empty ring), Sheet (lock icon).
3. The chat, taking most of the screen: consultant "What are you growing now, and what grew here before?"; farmer "Peas now. Lettuce before."; a row of three pea plant photos, one marked Blurry; consultant "Take the underside of a leaf, closer."; a small status line with a satellite icon "Checking NASA data for your field…".
4. A bottom sheet peeking up over the chat, showing the Pests section: title "Pest care · in progress", one line "Possible aphids. Beneficial insects: lacewings (Chrysoperla).", source "NASA POWER · degree-days".
5. Bottom: a message field "Write or send up to 10 photos" with a round camera button.
```
