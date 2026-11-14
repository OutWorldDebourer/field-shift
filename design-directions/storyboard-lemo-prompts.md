# Storyboard prompts in the Lemo-Opuscar spirit

Written 2026-11-14 for the owner to run in ChatGPT, after reading the two Lemo-Opuscar style guides the owner chose (`styles/satellite-briefing/STYLE.md` and `styles/living-screencast/STYLE.md` in the installed plugin). It fuses them: the Satellite Briefing grammar (real imagery, one continuous dive, hairline marks, one reading per screen) carries the opening and every map; the Mascot Screencast grammar (a real interface redrawn, window on wallpaper, cursor as the user, chapter pills, caption pills) carries the app. The owner asked for no mascot: the software's agent is the briefing's lock-on reticle.

Run everything in ONE ChatGPT conversation: the style bible first, then one shot per message. Save results here as `lemo-01.png` … `lemo-11.png`. These stills are AI-generated storyboards, not evidence; the real app and film use real licensed imagery.

## Style bible (send first)

```text
We are going to storyboard a short product film for a web app, one still frame per message, in this conversation. Every frame must follow this style bible exactly, so the frames read as one film. Reply "ready" and wait for shot 1.

THE PRODUCT
Field Shift, a web app for the NASA Space Apps Challenge 2026. A small farmer sends photos of the crop to an AI consultant; the app cross-checks them with NASA Earth observation data for that exact field and builds a pre-diagnosis (never a final diagnosis) with soil, pest-care, nutrition and crop-rotation options, plus a technical sheet for the agronomist. English UI text.

1. ESSENCE
- Two worlds in one film. Outside the app: a satellite briefing, real-looking satellite imagery of a real place with thin measurement lines drawn on the ground, one continuous dive from orbit to one field. Inside the app: a calm product walkthrough that looks like a real screen recording of a real interface, the app window floating on a wallpaper.
- Two actors, never a hand. The mouse cursor is the farmer. The software is a lock-on reticle: four thin corner brackets that snap onto whatever the software is reading (a field, a photo, a data row) with a slight overshoot. The same reticle lives on the satellite imagery and inside the app; it is the thread of the film.
- Two layers, two truths: the imagery is what the satellite saw; the hairlines and readings are what was measured. One finding per frame.
- Not a dark sci-fi HUD (no glowing wireframes, no invented chrome), not a slide deck (no header bars), not a dashboard collage, not a game.

2. MATERIALS
- Ground: top-down (nadir) or gently oblique satellite imagery of the place, never an illustration, never a 3D diorama.
- Daylight grade: the imagery is softly desaturated toward a pale silver-celeste haze, like an analyst's grade lifted into morning light; true colour returns only where the story turns.
- Marks: white hairlines 1 to 2 px with a faint dark halo so they read on light ground; dashed lines for history and plans; ticks, brackets, leader lines from a label to the exact spot it describes.
- Instruments stay in screen corners only: altitude readout, scale bar, imagery credit.
- The app: a real, believable interface redrawn crisp and sharp, in a window with a soft shadow floating on a wallpaper of a dawn sky (pale celeste fading to a warm peach horizon). Depth only where it guides attention: the pane not being read may be softly out of focus; text being read is always sharp.

3. COLOUR LOGIC
- Base: white and pale celeste; neutral silver-celeste ground.
- White means measured.
- One warm sunrise amber accent (around #FFBE55) for the single reading that matters in the frame; never decoration.
- One flat cartographic blue (around rgb(46,112,206)) only for the plan the data tests: the rotation proposal drawn as a dashed map layer.
- True colour is evidence: at orbit, and on the field when the recommended rotation turns it green.
- No rainbow false-colour ramps over the imagery.

4. TYPE AND CAPTIONS
- The reading: a light condensed grotesk, large (like Barlow Condensed Light).
- Instruments, dates and sources: a monospace (like IBM Plex Mono), e.g. "NASA POWER · T2M · 1984–2026".
- Labels and UI: a clean sans (like Source Sans 3); titles and the plan layer: a serif (like Source Serif 4).
- Labels attach to the ground with a leader line. The reading sits in the right third beside what it measures, label above, source below.
- Inside the app: a small chapter pill at the top left (01 to 05 with a name), a frosted-glass caption pill at the bottom centre with one short sentence, never over the element being discussed.
- Nothing tiny or unreadable.

5. FRAME GRAMMAR
- Every frame is a 16:9 film still.
- At key moments the subject fills about a third of the frame height.
- Allowed camera moves, shown as a still: oblique orbit with the planet's limb, dive toward the ground, locked nadir frame, push-in to a point, pull back to the full window, rack focus between panes, split-window comparison, push-in with a soft vignette on the one promise to remember.

6. NEVER
No NASA logo, flags or mission patches. No hands, no people, no mascot character. No lorem ipsum: every label is real and coherent across frames (field 2.1 ha, Huaral valley, Peru, 11.47°S 77.21°W; peas now, lettuce before). A tiny corner tag "SI-assisted visuals" on every frame.
```

## Shot 1: orbit at sunrise

```text
Shot 1. Oblique orbit at sunrise over the Pacific coast of South America: the planet's limb curves across the top with a thin atmosphere rim, the terminator line of dawn crossing the Andes, the coast in true colour. A searching reticle (four thin white brackets) hovers over central Peru, not yet locked. Bottom-left instrument in monospace: "ALT 420 km". Bottom-right imagery credit: "NASA Blue Marble". The title "Field Shift" in a quiet serif, centred, small.
```

## Shot 2: the dive

```text
Shot 2. The dive: the camera has descended and tilted toward nadir over the coastal desert valley of Huaral, Peru. A river cuts through dry hills to the ocean, a patchwork of small irrigated fields along it, everything in the silver-celeste daylight grade. A white leader line connects a label to the valley: "Huaral valley · 11.47°S 77.21°W". Instruments: "ALT 12 km" and a scale bar "1 km". Credit: "Landsat · USGS/NASA".
```

## Shot 3: lock-on

```text
Shot 3. Locked nadir frame on one field. The reticle has snapped onto a small field: its outline is printed as a white hairline polygon, the brackets frame it with a slight overshoot. Only this field keeps its true green; the rest stays in the silver-celeste grade. In the right third, the reading in large light condensed type: "2.1 ha", label above "YOUR FIELD", source below in monospace "Sentinel-2 L2A · 10 m". Scale bar "100 m".
```

## Shot 4: chapter 01, the field becomes the app

```text
Shot 4. The reticle opens out and becomes the frame of the app: a clean Field Shift app window floating on a dawn-sky wallpaper. Chapter pill top-left: "01 Your field". Inside the window: the same satellite view of the field, now in the app's map; a switch "My GPS | Example site" with Example site selected; buttons "Drop a point" and "Draw outline". The mouse cursor drags the last point of the field outline closed; a chip reads "2.1 ha · Huaral, Peru". Caption pill at the bottom: "Start with the field you farm."
```

## Shot 5: chapter 02, capture

```text
Shot 5. Push-in on the app's analysis workspace. Chapter pill: "02 Capture". Left two thirds: the chat with the consultant. Consultant: "What are you growing now, and what grew here before?" Farmer: "Peas now. Lettuce before." The cursor drops four pea-plant photos into the chat. The reticle snaps onto photo 3, tagged "blurry"; the consultant answers: "Take the underside of a leaf, closer." Right third: a column of six section buttons, all still empty dashed outlines: Your field, Soil, Pest care, Nutrition, Rotation, Technical sheet. Caption pill: "The consultant asks for what is missing."
```

## Shot 6: chapter 03, cross-check with NASA data

```text
Shot 6. Rack focus: the chat is softly out of focus; the right panel is sharp. Chapter pill: "03 Cross-check". A tag "▶▶ 4×" at the top right. The reticle runs down a list of six NASA readings as they land, each row with a small sparkline, a value and a monospace source: Temperature "+1.2 °C vs 40-year normal" (NASA POWER), Soil moisture "drier than normal" (SMAP · 9 km regional), Rain "3 mm this month" (GPM IMERG), Crop vigor "0.48" (Sentinel-2 NDVI · 10 m), Slope "2%" (SRTM), 2040 projection "+1.8 °C" (NEX-GDDP). The temperature value is the one amber reading. The section button "Your field" in the column is now filled. Caption pill: "Every number comes with its source."
```

## Shot 7: photo against satellite

```text
Shot 7. Split-window comparison inside the app. Left half: the farmer's close photo of pea leaves with yellowing lower leaves, framed by the reticle. Right half: the field on the satellite soil-moisture layer, regional pixels visibly larger than the field outline, framed by the same reticle, label "SMAP · 9 km regional context". Between them a thin white line and a verdict in the amber reading style: "Agree · supports water stress". Chapter pill: "03 Cross-check". Caption pill: "What the photo shows, checked from orbit."
```

## Shot 8: chapter 04, the field dashboard

```text
Shot 8. Pull back to the full app window showing the field dashboard. Chapter pill: "04 Dashboard". Left: a large map of the field in the daylight grade with the outline in a white hairline and a layer switch (Crop vigor · Soil moisture · Temperature · Slope). Top: six calm metric tiles, each with value, a plain-words verdict and a monospace source. Centre: a line chart "This season vs 40-year normal", a pale shaded normal band and this season's line rising above it in amber. Bottom: a twelve-month crop calendar strip with pea, lettuce and bare-soil periods. Right: three alerts (Heat stress next 7 days · Aphid generation window · Irrigation deficit). Caption pill: "Your field, read from space, season by season."
```

## Shot 9: chapter 05, pre-diagnosis

```text
Shot 9. Push-in with a soft vignette on the one promise to remember. Chapter pill: "05 Pre-diagnosis". A card: label "PRE-DIAGNOSIS · NOT A DIAGNOSIS", serif headline "Likely water stress with early aphid pressure", a confidence bar at "Medium", and the line "Confirm with your agronomist." Around it, slightly out of focus, the section column now filled: Soil (green manure, Trichoderma-enriched compost, no residue piles at the field edge), Pest care (lacewings Chrysoperla, Beauveria), Nutrition. Caption pill: "A starting point for your agronomist, not a verdict."
```

## Shot 10: the rotation plan on the ground

```text
Shot 10. Back on the satellite field, locked nadir frame. The plan layer appears as a flat cartographic blue dashed overlay on the field, in a serif label: "Strategy B · green manure, then lettuce". Three small serif tags beside the field compare the strategies: "Current · pea → lettuce", "A · add a legume", "B · green manure (best fit: improve soil)". Inside the outline, the field regains a deeper true green where the story turns. A white range line with ticks runs along a 12-month timeline under the field. Caption pill: "Three ways forward, compared on your own ground."
```

## Shot 11: the sheet, and the climb back

```text
Shot 11. Two moments in one frame: on the left, the app window with the cursor clicking "Export technical sheet (PDF)" and an A4 sheet sliding out, showing a header "Field Shift · Technical sheet", a NASA readings table and a sources list; on the right, the camera already climbing back to orbit, the valley shrinking under the reticle, instrument "ALT 120 km". End line in a quiet serif: "Field Shift · Team InnoHuando · NASA Space Apps Challenge 2026".
```
