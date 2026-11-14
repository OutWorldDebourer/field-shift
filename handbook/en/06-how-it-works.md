# 6. How it works and how to run it

Built during the hackathon (2026-11-14/15). Code: `backend/` (Rust), `frontend/` (TypeScript), `infra/` (MariaDB).

## The flow

1. **Field.** The farmer uses their GPS, an example site (Huaral valley, Peru), a place search, a dropped point or a drawn outline. The outline's area is computed on the sphere.
2. **Conversation.** The consultant (Claude, model `claude-opus-5-5`, via the Anthropic API) asks what grows now and what grew before, the water source and the last fertilizer, one or two questions at a time, and guides photos (up to 10 per message). "I don't know" is recorded as a declared gap.
3. **Analyst stage (evidence package).** The backend reads, in parallel, NASA POWER (climate since 1981 and its 1985–2024 normal, FAO-56 reference evapotranspiration, water balance, degree-days and pest generations), Sentinel-2 and Landsat crop vigor for the outline (Microsoft Planetary Computer), NASADEM slope, SMAP soil moisture, IMERG rain, GRACE water storage and the NEX-GDDP-CMIP6 projection. Every value carries a source, a date, a resolution, a licence and a scale badge: **field-level** (10–30 m) or **regional context** (POWER ~50 km, SMAP 9 km, IMERG 10 km, GRACE ~300 km).
4. **Crop knowledge.** Any crop the farmer names is normalized to a crop record (names, scientific name, family, type: annual, semi-perennial or perennial). A verified seed knowledge base answers known crops instantly; other crops get a trait card researched with web search, every value with its citation, labelled "AI-researched, verify".
5. **Advice.** Crop type decides the advice: annuals get three rotation strategies (the current one included) scored on soil, water, pest break, income and risk; semi-perennials get renewal and interim cover; perennials never get "rotate the main crop" but inter-row cover, intercropping, residue management and biological control. Soil, pest care (beneficial insects and fungi such as *Chrysoperla*, *Trichoderma*, *Beauveria*) and nutrition follow the agronomist interview.
6. **Consultant answers** cite only numbers from the evidence package, in plain words first. Everything is a **pre-diagnosis**; the farmer is sent to their agronomist.
7. **Export.** A printable technical sheet (save as PDF) with every reading, source, date and licence; a CSV of all series; a read-only share link.

If the live model is unavailable or a visitor reaches the rate limit (30 live turns per hour, 400 per day overall), the consultant switches to a clearly labelled **scripted demo mode** that never pretends to read photos.

## Run it locally

```bash
docker compose -f infra/compose.yaml --env-file infra/.env up -d
```

```bash
cd backend && cargo run --bin fieldshift
```

```bash
cd frontend && npm install && npx vite
```

Open http://localhost:5173 (index) and http://localhost:5173/app.html (app). The backend reads API keys from `env.txt` at the project root (`api.nasa.gov`, `earthdata`, `anthropic` lines) and database settings from `infra/.env`; neither file is in git.

## Services and endpoints (all under `/api`)

| Area | Endpoints |
|------|-----------|
| Health | `GET /health` |
| Analyses | `POST /analyses`, `GET /analyses/examples`, `GET /analyses/{id}`, `POST /analyses/{id}/evidence` |
| Consultant | `POST /consultant/{id}/message` (multipart: text, lang, photos) |
| Climate | `GET /power/summary`, `/power/series`, `/power/monthly` |
| Imagery | `GET /imagery/ndvi`, `/imagery/scenes`, `/imagery/terrain`, `/imagery/snapshot.png` |
| Earthdata | `GET /earthdata/smap`, `/earthdata/imerg`, `/earthdata/grace` |
| Projection | `GET /climate/projection` |
| Crops | `GET /crops/suggest`, `/crops/resolve`, `POST /crops/advice` |
| Export | `GET /export/{id}/sheet`, `/export/{id}/series.csv`, `POST /export/{id}/share` |

## Iteration 2 (2026-11-14)

- **Guided topics.** The consultant works through five topics in order (your field, soil, pest care, nutrition, rotation); each section turns "ready" only when its topic is done, the chat shows the next topic and what it still needs, and a short confirmation marks each completed topic. When all are done, the consultant says so and points to the technical sheet.
- **Greeting.** The first message says where the field is (latitude, longitude, area) and that the data describe the plot, not the person.
- **Forecast, outlook, projection (NASA only).** Next 1–10 days from NASA GEOS-FP (a forecast); the next 1–6 months as an estimate from NASA POWER normals with two scenarios (back to normal / as warm as now); 2031–2050 from NEX-GDDP-CMIP6 (a projection). The dashboard lets the farmer change each horizon.
- **Voice dictation.** A microphone fills the message box without sending; engines: Gemini (default, `POST /api/consultant/transcribe`) or the browser's speech recognition.
- **Photos at full resolution.** Kept up to 12 MB; the model also receives the four quarters of each large photo at native resolution so small insects stay visible.
- **Technical sheet.** Back link to the analysis, farmer photos, satellite picture, forecast and outlook, a glossary annex, and a ZIP package with the originals.

## Iteration 3 (2026-11-14)

- **Documents.** The chat accepts PDFs (up to 15 MB) besides photos. A farmer can attach the technical sheet of an earlier visit: the consultant reads it, says what it took from it and its date, records the answers and completes the topics it covers; NASA readings are always fresh.
- **Load information.** A form with the answers each topic needs (crop now and before, irrigation, soil, organic matter, residues, pests seen, recent products, last fertilizer, soil test, next crop, harvest use; "I don't know" allowed). On the example site it opens pre-filled with labelled example answers, so judges see every section filled. `POST /api/analyses/{id}/intake`.
- **Organic tab.** Biological and ecological pest management: natural enemies and beneficial microorganisms (insects, mites, fungi, bacteria) with their published sources, those matching the reported pests first (matched across the whole crop knowledge base); practices that keep them working; when to act from NASA degree-days and the GEOS-FP forecast; what harms them. No doses: those are the agronomist's call.
- **What the farmer told us** appears in the Your field section and on the technical sheet.

