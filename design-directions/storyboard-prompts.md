# Storyboard prompts: the web app scene by scene

Written 2026-11-14 for the owner to run in ChatGPT. Run them in ONE conversation, in order: the setup message first, then one scene per message, so the style stays consistent. Save results here as `story-01.png` … `story-09.png`.

## Setup message (send first)

```text
We are going to create a storyboard of a web application, one screen per image, in this same conversation. Keep the exact same visual style, colors, fonts, icons and app name in every image so they look like one product. I will send one scene per message. Each image is a single flat UI screen, full-frame, with no device mockup, no browser chrome and no people.

The app: Field Shift, a web app for the NASA Space Apps Challenge 2026. Small farmers chat with an AI consultant, send photos of their crop, and the app cross-checks them with NASA Earth observation data for their exact field. The result is a pre-diagnosis (never a final diagnosis) with soil, pest-care, nutrition and crop-rotation options to confirm with an agronomist, plus a technical sheet. English UI text.

Style for every image: light, clean, professional and spacious; white and soft sky blue as the main colors; one warm accent color only for warnings and key numbers; clear readable typography; real satellite imagery where maps appear; small source labels under every number (e.g. "NASA POWER · 1984–2026"). No NASA logo, no flags, no mission patches.

Reply "ready" and wait for the first scene.
```

## Scene 1: landing page (landscape 3:2)

```text
Scene 1, landscape 3:2, desktop landing page. A satellite view of an irrigated valley at sunrise as the hero background, fading to white behind the text. Headline: "Photos from your field. NASA data from above. A pre-diagnosis you can verify." Buttons: "Start analysis" (primary) and "Try an example site" (secondary). Below: a three-step "How it works" strip (Capture · Cross-check with NASA data · Pre-diagnosis), a row of the NASA datasets used (NASA POWER, SMAP, GPM IMERG, Sentinel-2, SRTM, GRACE-FO), and a footer line "A support tool. It does not replace your agronomist." A language switch EN | ES in the top bar.
```

## Scene 2: choose the field (portrait 2:3, phone)

```text
Scene 2, portrait 2:3, phone screen. Title "Where is your field?". A switch with two options: "My GPS" and "Example site" (selected). A list of example sites: Huaral valley, Peru · Central Valley, California · Iowa corn belt. A satellite map below with a field outline being drawn point by point (five dots joined by a line), a label "2.1 ha". Two small mode buttons: "Drop a point" and "Draw outline" (selected). Primary button at the bottom: "Use this field".
```

## Scene 3: the consultant asks (portrait 2:3, phone)

```text
Scene 3, portrait 2:3, phone screen. The analysis workspace on a phone. Top: field chip "2.1 ha · Huaral". Under it, a horizontal row of six small round section buttons with progress rings and labels: Field, Soil, Pests, Nutrition, Rotation, Sheet (only Field partly filled). The chat: consultant "What are you growing now, and what grew here before?"; farmer "Peas now. Lettuce before."; farmer sends four photos of pea plants; consultant "Photo 3 is blurry. Take the underside of a leaf, closer." Bottom: message field "Write or send up to 10 photos" with a camera button.
```

## Scene 4: cross-checking with NASA data (portrait 2:3, phone)

```text
Scene 4, portrait 2:3, phone screen. The consultant is working: a message "Checking NASA data for your field" with a checklist that fills in, each line with a status icon and a source: Temperature +1.2 °C vs 40-year normal (NASA POWER) ✓, Soil moisture drier than normal (SMAP) ✓, Rain 3 mm this month (GPM IMERG) ✓, Crop vigor 0.48 (Sentinel-2) ✓, Slope 2% (SRTM) ✓, Climate projection 2040 (loading). Below, a card "Photo vs satellite": a thumbnail of yellowing lower leaves next to "Soil drier than normal" with the verdict "Agree: supports water stress".
```

## Scene 5: the field dashboard (landscape 3:2, desktop)

```text
Scene 5, landscape 3:2, desktop dashboard of one field. Left: a large satellite map of the field outline with a layer selector (Crop vigor NDVI · Soil moisture · Temperature · Slope) and a scale note "Field-level: 10 m · Regional: 9 km". Top row: six metric tiles, each with a value, a plain-words verdict and a source: Temperature +1.2 °C (warmer than normal), Soil moisture (drier than normal), Rain 3 mm (below normal), Crop vigor 0.48 (moderate), Water balance −42 mm (deficit), Pest pressure +2 aphid generations. Center: a line chart "This season vs 40-year normal" with a shaded normal band and this season's line above it. Right: an alerts panel (Heat stress next 7 days · Aphid generation window · Irrigation deficit) and a small "2040 projection: +1.8 °C" card. Bottom: a 12-month crop calendar strip showing pea, lettuce and bare-soil periods.
```

## Scene 6: the field dashboard on a phone (portrait 2:3)

```text
Scene 6, portrait 2:3, phone version of the same dashboard. A small map of the field at the top with the layer selector as chips. Then the six metric tiles in a two-column grid, each with value, verdict and source. Then the "This season vs 40-year normal" chart, compact. Then one alert card "Aphid generation window: weeks 2–4". Bottom navigation with: Chat, Dashboard (selected), Sections, Sheet.
```

## Scene 7: pre-diagnosis and sections (portrait 2:3, phone)

```text
Scene 7, portrait 2:3, phone screen. A label "PRE-DIAGNOSIS · NOT A DIAGNOSIS", headline "Likely water stress with early aphid pressure", a confidence meter at "Medium", and "Confirm with your agronomist". Below, three expandable section cards: Soil ("Low organic matter: add green manure and Trichoderma-enriched compost; do not pile crop residues at the field edge"), Pest care ("Release lacewings (Chrysoperla); Beauveria as a biological option; monitor leaf undersides weekly"), Nutrition ("Likely nitrogen demand after lettuce; a legume can supply part of it"). Each card ends with its sources and one "Information gap" line.
```

## Scene 8: rotation comparison (landscape 3:2, desktop)

```text
Scene 8, landscape 3:2, desktop screen "Compare rotation strategies". Three equal columns: Current (pea then lettuce), Strategy A (add a legume), Strategy B (green manure). Each column has a small 12-month timeline of crops, then the same rows with simple scores and short notes: Soil health, Water use, Pest cycle break, Extra income, Risk. Then for each: "Why suggested", "Evidence" (with sources), "Information gaps", "Check locally". Strategy B carries a "Best fit for your priority: improve soil" tag. A button "Add to technical sheet".
```

## Scene 9: the technical sheet (portrait, A4 page)

```text
Scene 9, portrait A4 document page, the exported technical sheet as a clean PDF. Header: "Field Shift · Technical sheet · Pre-diagnosis", field ID, date, location and area (2.1 ha, Huaral, Peru), coordinates. Sections: Crop history; Photos (three small thumbnails); NASA readings table (variable, value, normal, source, date); Pre-diagnosis with confidence; Soil, Pest care, Nutrition summaries; Rotation options table; Information gaps; Sources list (NASA POWER, SMAP, GPM IMERG, Sentinel-2, SRTM, local references). Footer: "Support tool. Final decision by a qualified agronomist." and a small signature box "Reviewed by".
```
