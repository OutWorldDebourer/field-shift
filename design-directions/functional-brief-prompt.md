# Functional brief prompt (design-neutral)

Written 2026-11-14 at the owner's request: a prompt that describes what Field Shift does, with no visual direction, to hand to any design tool.

```text
I need you to design a web application. Below is only what it does and who uses it. I am deliberately giving you no visual direction: propose the layout, look and interaction you think serve these users best, and explain your choices.

PRODUCT
Field Shift is a mobile-first web app (it must also work on tablet and desktop) for the NASA Space Apps Challenge 2026, challenge "Field Shift: Adapting Farms with NASA Data". It helps small farmers decide how to rotate crops and care for their soil and plants as the climate changes. The farmer talks with an AI consultant, sends photos of the crop, and the app cross-checks what it sees with NASA Earth observation data for that exact field. The result is a pre-diagnosis (never a final diagnosis) with options to review with an agronomist, plus a technical sheet the agronomist can verify.

USERS
1. Small farmers (fields of 1 to 5 hectares, e.g. the Huaral valley in coastal Peru) using a phone outdoors, often in bright sun, with little technical background.
2. The agronomist or extension engineer who receives the exported technical sheet and makes the final decision.
3. Contest judges, mostly in the United States, who will test the app without logging in, using example locations and sample data.

LANGUAGE
English by default, with a visible switch to Spanish. Everything switches, including the consultant's replies and the exported sheet.

PAGES AND FUNCTIONS

1. Landing page
- Explains in a few seconds what the app does: photos from the field + NASA data from above = a pre-diagnosis you can verify.
- Primary action: Start analysis. Secondary action: Try an example site (for judges and first-time visitors; loads a ready-made farm with location, crop history and photos).
- Short "how it works" in three steps: capture, cross-check with NASA data, pre-diagnosis.
- Lists the NASA datasets used and what each one tells the farmer.
- States clearly that the tool supports, and does not replace, the agronomist.
- May include a background video showing a satellite view of a valley at sunrise (must be pausable, with a still image fallback).

2. Choosing the field
- Two modes with a switch: My GPS (the phone's real location) or Example site (a list of preset locations: a Peruvian valley and one or two U.S. farm regions).
- The user can drop a single point on a map or draw the field's outline point by point; the app shows the area in hectares.
- The user can change the field at any time.

3. Analysis workspace (the core of the app)
It combines two things that are visible at the same time:

a) A chat with the AI consultant
- The consultant behaves like a friendly agronomy consultant. It asks for whatever is missing, one or two questions at a time, in plain words: current crop, crops of the last one or two seasons, water source (river, well, rain; furrow or drip), and the farmer's priority (save water, extra income, improve the soil). Optional: soil type, equipment, market.
- "I don't know" is always an acceptable answer; it becomes an information gap shown in the results, not a blocker.
- The farmer can send up to 10 photos per message. The consultant guides the photos: asks for specific shots (whole plant, underside of a leaf, soil surface), says when a photo is blurry or too far, and asks for another angle.
- While working, the consultant shows what it is doing (for example: reading temperature data, checking soil moisture).

b) A set of result sections that fill in as the conversation progresses
The sections come from an interview with a local agronomist plus the challenge requirements:
- Your field (NASA data): temperature versus the 40-year normal, heat stress, frost risk, soil moisture, rainfall and extreme rain, evapotranspiration and water balance, crop vigor (NDVI) and bare-soil periods, slope and erosion risk, and a climate projection for the coming decades.
- Soil: organic matter, organic and green manures, beneficial microorganisms (for example Trichoderma-enriched compost), crop residue management (avoid leaving residue piles that become infection hotspots).
- Pest care (integrated pest management): likely pests, how warmer temperatures speed up pest generations, preventive actions, and beneficial insects and fungi for biological control (for example lacewings, parasitoid wasps, Beauveria, Trichoderma).
- Nutrition: what the crop and soil likely need.
- Rotation: at least three rotation strategies compared side by side, including the farmer's current rotation. For each one: why it was suggested, the evidence behind it, benefits (soil, water, pests, income), information gaps, and risks that need local review.
- Technical sheet: becomes available when enough information exists.
Each section shows its state: empty (and what is missing), in progress, ready, or ready with gaps. Tapping a section that is missing information makes the consultant ask for exactly that. Each section can be opened to read its full content.

4. Evidence and sources
- Every number shows where it comes from and its date (for example: NASA POWER, daily temperature, 1984-2026).
- The app shows how photo observations and satellite data agree or disagree (for example: yellowing lower leaves in the photo + soil drier than normal from satellite = supports water stress).
- The app is honest about resolution: some satellite data describes a whole region (kilometers), other data describes the field itself (meters); the user must be able to tell which is which.
- The pre-diagnosis shows a confidence level and always reminds the user to confirm with an agronomist.

5. Export and sharing
- Technical sheet as a PDF: field data, location and area, crop history, photos, NASA readings with sources and dates, the pre-diagnosis with confidence, the sections above, the three rotation options, information gaps, and the full list of sources.
- Raw data download (CSV) of the NASA series used.
- A share link for the report.

DATA SOURCES (for content, not for styling)
NASA POWER (temperature, rain, humidity, radiation, evapotranspiration), SMAP (soil moisture), GPM IMERG (precipitation), Landsat and Sentinel-2 (vegetation index, bare soil), MODIS (vegetation trends), SRTM (elevation and slope), GRACE/GRACE-FO (long-term water storage), a NASA climate projection dataset, plus local crop and pest knowledge.

STATES TO DESIGN
First visit, example site loaded, consultant asking, photos uploading, photo rejected as blurry, data loading, section empty / in progress / ready / with gaps, no internet, data unavailable for a location, export ready.

CONSTRAINTS
- Readable on a phone outdoors in bright light.
- No login required.
- No NASA logos, flags or mission insignia.
- AI-generated images or video in the product must carry a small visible "SI-assisted" mark.

DELIVERABLE
Propose the screens for: landing page, field selection, analysis workspace (phone and desktop), one opened section (Rotation), and the exported technical sheet. Explain the reasoning behind your layout.
```
