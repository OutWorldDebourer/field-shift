# Index and system prompts, from the approved orbit frame

Written 2026-11-14. The owner approved the look of storyboard shot 1 (orbit at sunrise over the Peruvian coast, reticle, italic serif title, monospace instruments). These prompts rebuild the landing page (index) and the app around it, for the owner to run in ChatGPT, one conversation, style bible first. Save results as `index-01.png` … `index-06.png`, `system-01.png`, `system-02.png`.

Concept: the page is a sunrise. The top of the index is space before dawn; scrolling dives toward the valley and the light rises, so the app itself lives in daylight (white and celeste, readable outdoors). In the build, the globe is a live Three.js scene with real NASA textures; these images are only storyboards.

## Style bible (send first, attach the approved orbit image)

```text
Attached is a frame I love. We will design a website in exactly this visual language, one image per message, in this conversation. Reply "ready" and wait.

THE PRODUCT
Field Shift, a web app for the NASA Space Apps Challenge 2026. A small farmer sends crop photos to an AI consultant; the app cross-checks them with NASA Earth observation data for that exact field and builds a pre-diagnosis (never a final diagnosis) with soil, pest-care, nutrition and crop-rotation options, plus a technical sheet for the agronomist. English text.

THE VISUAL LANGUAGE (taken from the attached frame)
- Cinematic Earth from orbit: true-colour planet with real terrain relief, a thin glowing blue atmosphere rim, deep navy-black space, a warm sunrise flare breaking over the limb. Photographic, calm, awe-inspiring, never cartoon, never sci-fi neon.
- The thread of the design is a lock-on reticle: four thin white corner brackets that snap onto what matters.
- Type: an elegant italic serif for titles (like Source Serif 4 Italic), a clean monospace in white for instruments and data (like IBM Plex Mono), a clean sans for body text (like Source Sans 3). Small, spacious, precise.
- Instruments live in the corners only, in monospace: altitude, scale bar, imagery credit. A tiny "SI-assisted visuals" tag top-left.
- Thin white hairlines and leader lines connect labels to the exact place on the ground they describe.
- One warm sunrise amber (around #FFBE55) only for the single key number on screen. White means measured.
- Interface panels are frosted glass: translucent white with a soft blur of the Earth behind, thin white borders, generous padding, no heavy shadows.
- The page is a sunrise: top sections are space before dawn; lower sections descend toward the ground and grow brighter, ending in daylight white and pale celeste.

NEVER: NASA logos, flags or mission patches; people or hands; glowing wireframes; cluttered dashboards; lorem ipsum. Keep labels coherent: Huaral valley, Peru, 11.47°S 77.21°W, field 2.1 ha, peas now, lettuce before.
```

## Index 1: the first viewport (landscape 16:9)

```text
Index 1, landscape 16:9, the website's first screen. Full-bleed: the same orbit view as the attached frame, sunrise over the Andes, as a live background. Top bar, thin and transparent: italic serif wordmark "Field Shift" at left; links "How it works", "NASA data", "About" in small sans; a language switch "EN | ES" at right. Left third, over dark space: a large italic serif headline "See your field from orbit." and one line of sans text "Send photos of your crop. We cross-check them with NASA Earth data and return a pre-diagnosis your agronomist can verify." Two buttons: a solid white button "Start analysis" and a frosted-glass outline button "Try an example site". Right side, floating in front of the planet with a soft depth-of-field blur behind it: a frosted-glass phone showing the app in use (a chat bubble "Take the underside of a leaf, closer.", two pea-plant photos, and three data rows with monospace sources: Temperature +1.2 °C · NASA POWER, Soil moisture drier than normal · SMAP, Crop vigor 0.48 · Sentinel-2). The reticle hovers over the coast of Peru below the phone. Corners: "ALT 420 km" bottom-left, "NASA Blue Marble" bottom-right, "SI-assisted visuals" top-left. A tiny scroll cue "Scroll to descend" at the bottom centre.
```

## Index 2: the dive, how it works (landscape 16:9)

```text
Index 2, landscape 16:9, the next section as the user scrolls: the camera has dived toward the coast; the Huaral valley is in view from about 40 km, the sky glow brightening. The reticle has locked onto the valley with a white leader line to the label "Huaral valley · 11.47°S 77.21°W". Over the lower half, three frosted-glass cards side by side, numbered in monospace: "01 Capture" (send photos and tell the consultant what you grow), "02 Cross-check" (your photos against NASA data for your exact field), "03 Pre-diagnosis" (soil, pest care, nutrition and rotation options to confirm with your agronomist). Section title in italic serif: "How it works". Instrument "ALT 40 km" bottom-left, scale bar, credit "Landsat · USGS/NASA" bottom-right.
```

## Index 3: NASA data on the ground (landscape 16:9)

```text
Index 3, landscape 16:9. Locked top-down view of the irrigated valley in early morning light: river, dry hills, a patchwork of small fields. One field is outlined in a thin white hairline with the reticle on it, labelled "2.1 ha". Six white leader lines run from that field to six small monospace labels placed around it, each a NASA dataset with what it tells the farmer: "NASA POWER · temperature, rain, heat stress", "SMAP · soil moisture (9 km, regional)", "GPM IMERG · rainfall and storms", "Sentinel-2 / Landsat · crop vigor, bare soil (10–30 m)", "SRTM · slope and erosion", "GRACE-FO · long-term water storage". One label in amber: "+1.2 °C vs 40-year normal". Section title in italic serif top-left: "Read from space, field by field." Light is clearly brighter than the previous sections.
```

## Index 4: what you get (landscape 16:9)

```text
Index 4, landscape 16:9, daylight now: the satellite view of the field is softly blurred and brightened behind a row of five frosted-glass cards, each with a thin line icon, a title in italic serif and two short lines: "Soil" (organic and green manures, beneficial microorganisms such as Trichoderma, residue management), "Pest care" (likely pests and beneficial insects and fungi such as lacewings and Beauveria), "Nutrition" (what the crop and soil likely need), "Rotation" (three strategies compared, your current one included), "Technical sheet" (a cited PDF for your agronomist). Above them, in italic serif: "Built with agronomists, for the field." A small line under the cards: "A pre-diagnosis, not a diagnosis. Always confirm with your agronomist."
```

## Index 5: dashboard preview (landscape 16:9)

```text
Index 5, landscape 16:9, daylight white and pale celeste. A large frosted-glass browser-like panel previews the app's field dashboard: on the left a top-down map of the field with a white hairline outline and a layer switch (Crop vigor · Soil moisture · Temperature · Slope); across the top six calm metric tiles with value, plain-words verdict and monospace source; a line chart "This season vs 40-year normal" with a pale band and an amber line above it; a twelve-month crop calendar strip; an alerts list. Section title in italic serif: "Your field, season by season." A short caption: "Every number comes with its source and date."
```

## Index 6: final call to action and footer (landscape 16:9)

```text
Index 6, landscape 16:9. The camera has climbed back to orbit at full sunrise: the planet below, warm light across the limb, the reticle resting on Peru. Centred: italic serif headline "Start with the field you farm.", two buttons "Start analysis" (solid white) and "Try an example site" (frosted glass). Below, a quiet frosted-glass footer strip in small sans and monospace: "A support tool. It does not replace your agronomist." · "Team InnoHuando · NASA Space Apps Challenge 2026" · "Data: NASA POWER, SMAP, GPM IMERG, Landsat, Sentinel-2 (contains modified Copernicus Sentinel data), SRTM, GRACE-FO" · "Sources and licences". Corner instruments as before.
```

## System 1: the analysis workspace, desktop (landscape 16:9)

```text
System 1, landscape 16:9, the app itself after pressing "Start analysis", in daylight so it is readable outdoors. Background: the top-down view of the farmer's valley, brightened and softly blurred, the field outlined in a white hairline with the reticle. Thin top bar: italic serif "Field Shift", a frosted chip "2.1 ha · Huaral, Peru" with a switch "My GPS | Example site", "EN | ES". Left two thirds: a frosted-glass chat with the consultant: "What are you growing now, and what grew here before?" / "Peas now. Lettuce before." / four pea-plant photos, photo 3 framed by the reticle and tagged "blurry" / "Take the underside of a leaf, closer." / a monospace status "Reading NASA POWER · SMAP · Sentinel-2…"; a message field "Write or send up to 10 photos" with a camera button. Right third: a frosted-glass column of six section buttons with thin progress rings and states: Your field (ready, with three monospace readings), Soil (in progress), Pest care (in progress, amber note "needs a closer photo"), Nutrition (empty), Rotation (empty), Technical sheet (locked). A "Dashboard" tab beside "Chat" at the top of the chat panel.
```

## System 2: the analysis workspace, phone (portrait 9:19.5)

```text
System 2, portrait phone screen, same app and language. Background: the farmer's field from above, brightened and blurred. Top: italic serif "Field Shift", chip "2.1 ha · Huaral", "EN | ES". Under it, a horizontal row of six small round section buttons with thin progress rings and labels: Field (full), Soil (half), Pests (half, amber dot), Nutrition, Rotation, Sheet (lock). The frosted-glass chat fills the screen: the consultant's question, the farmer's answer, three pea-plant photos with the reticle on the blurry one, the consultant's request for a closer photo, a monospace status line "Checking NASA data for your field…". Bottom: a message field "Write or send up to 10 photos" and a round camera button.
```
