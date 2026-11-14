# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Backend in Rust (owner's standing preference). Frontend in HTML, CSS and TypeScript as a mobile-first progressive web app; the remaining choices (map library, build tool) were delegated to the agent and are recorded in the architecture docs when made.

## Users

- **Primary:** smallholder farmers in the Huaral valley (Lima region, coastal Peru), irrigated desert plots of roughly 1 to 5 ha, often with little technical or geospatial background, using a phone in the field.
- **Secondary:** the agronomist or extension engineer who receives the farmer's exported technical sheet and makes the final call.
- **Evaluators:** NASA Space Apps judges (local event, then global judges largely in the United States) who test with example sites and synthetic farmer inputs, without a login.

## Product Purpose

Field Shift is a pre-diagnosis tool, never a diagnosis. The farmer sets the field (GPS or an example site, a point or a drawn polygon), tells an AI consultant what is growing and what grew before, and sends photos. The tool combines NASA Earth observations with crop and pest knowledge to compare at least three rotation strategies, the current one included, each with why it was suggested, evidence, benefits, information gaps and risks for local review. It covers soil, pest management (including beneficial insects and fungi such as Trichoderma) and nutrition, and exports a cited technical sheet for the engineer. Success: the farmer understands their options in plain words, and the engineer gets the evidence to decide.

## Positioning

The bridge from NASA climate data to pest pressure: daily temperature from NASA POWER turns into degree-days and estimated pest generations against a 40-year baseline, which then drives rotation timing and biological-control windows. Every number in the product cites its source and date.

## Operating Context

- Used outdoors on a phone, often in bright sun, sometimes with weak connectivity.
- Conversation with the consultant in the farmer's language; the consultant asks for missing data (current crop, previous crops, water source) and guides photos (angle, closer shot), up to 10 photos per message. "I don't know" is a valid answer that becomes a declared information gap.
- A two-stage harness: an analyst stage gathers NASA and catalog data into a fixed evidence package; the consultant model presents it and may only cite numbers from that package.
- Judges evaluate a public link with no login during the judging period (through January 2027).

## Capabilities and Constraints

- Location: real GPS or example site switch; point or polygon field drawing with area.
- Data: NASA POWER (no login), Sentinel-2 / Landsat via Microsoft Planetary Computer, SMAP and GPM as regional context; resolution limits are shown, not hidden.
- Export: technical sheet (PDF) with citations, raw data (CSV), share link.
- Contest rules: all submission material in English; every source cited with its license; original work open source (Apache 2.0 by default); AI use disclosed, and AI-generated images or video carry a visible watermark; no NASA logos, flags or mission identifiers in generated content; no images of people under 18.
- Undecided: final crop list for the pilot catalog; AI billing route (Anthropic API key or paid Google Cloud Vertex AI).

## Brand Commitments

- Name: Field Shift (team InnoHuando).
- English is the default language everywhere, with a Spanish switch.
- Owner's binding visual constraints: clean, corporate, white and celeste, spacious, evoking NASA without its marks, with sunrise warmth. References named by the owner: Lemo-Opuscar "Satellite Briefing" (lightened to daylight) and "Mascot Screencast" (a pointer in place of a mascot), and Lemo-Opuscar's typography and graphic line.

## Evidence on Hand

- Interview with agronomist Ing. Cenia: `Entrevista/Ing. Cenia.txt`.
- Owner's own crop photos (pea / arverjita), location to be provided.
- Official challenge text and rules: `handbook/`.
- Absent and not to be fabricated: farmer testimonials, yield results, partner institutions, usage numbers.

## Product Principles

1. Support, never replace: the farmer and the engineer decide; the tool says where human judgment matters.
2. Every number has a source and a date.
3. Plain words first, the figure second.
4. Honest about resolution: regional satellite context is never presented as a field measurement.
5. Missing information is shown as a gap, not hidden or guessed.

## Accessibility & Inclusion

Readable outdoors in bright light on a phone; plain language for users with little technical background; bilingual English and Spanish.
