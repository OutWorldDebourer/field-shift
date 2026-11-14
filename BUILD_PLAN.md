# Build plan (autonomous run)

Written 2026-11-14 before the hackathon start. The owner (Masahiko, team InnoHuando) delegated the build to an unattended agent run starting **2026-11-14 09:00 Peru time (UTC-5)**, the official start of the NASA Space Apps Challenge. Submission closes **Sunday 2026-11-15 23:59** local time. The owner may be asleep: do not wait for answers; take the documented default, log it in "Decision log" below, and keep going.

## Read first (in this order)

1. `README.md`, `PRODUCT.md` (product truth), `handbook/en/01-challenge-brief.md`, `handbook/en/02-rules-and-submission.md`, `handbook/en/03-architecture.md`.
2. `research/01_api_tests.md`: every data source with the exact URLs that worked, quirks and values.
3. `Entrevista/Ing. Cenia.txt` (Spanish): the agronomist interview that defines the result sections.
4. `design-directions/approved-index-hero.webp` and `approved-orbit-shot1.webp`: the approved visual direction. `design-directions/index-prompts.md` and `app-prompts.md` describe the full index and app in that language.

## Standing rules

- **Language:** all code, UI text, docs and commits in English by default; every UI string also in Spanish behind an EN | ES switch. Talk to the owner in Spanish.
- **Modular:** small files, services split by business area, no monolithic files. Every folder gets an ultra-short English `README.md` (what each file does). Update the folder README in the same change as the files.
- **Rust backend** (owner's standing preference), **MariaDB 11.4** (`infra/compose.yaml`, credentials in `infra/.env`), **TypeScript frontend** (Vite), **Three.js** globe.
- **Secrets:** `env.txt` holds `api.nasa.gov <key>`, `earthdata <token>` and, if the owner added it, `anthropic <key>`. Read them at runtime; never print, log, commit or paste them. `infra/.env` likewise.
- **Contest rules:** cite every source with its license (keep `handbook/en/07-references.md` + `es/` mirror current); non-NASA material must be open; no NASA logos, flags or mission insignia; log every AI assistance in `AI_USE.md`; AI-generated images or video shipped in the product carry a visible "SI-assisted" mark; satellite imagery shown as evidence must be real and licensed (Blue Marble, VIIRS, Landsat, Sentinel-2 with "Contains modified Copernicus Sentinel data [year]"), never AI-generated.
- **Honesty about scale:** every reading carries a source, a date and a badge "field-level" (Sentinel-2/Landsat 10-30 m) or "regional context" (POWER ~50 km, SMAP 9 km, IMERG 10 km, GRACE ~300 km).
- **Pre-diagnosis, never diagnosis:** the UI and the consultant always say so and send the user to an agronomist.
- **Git:** commit after each working milestone with a clear message ending `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, push to `origin` (private repo `OutWorldDebourer/field-shift`). Never make the repo public, never force-push.
- **Never without the owner:** creating accounts, entering payment data, buying, claiming the GoDaddy/Porkbun domain, creating Google Cloud resources, making anything public, sending messages. Leave these for the morning in "Needs the owner".
- **Verify, don't assume:** run what you build (cargo test / cargo run, npm run build, the browser pane) before marking a step done. Use the `impeccable` skill for UI work and its finish review; the approved hero image stands in as the approved comp.

## Milestones (in order; ship each before starting the next)

1. **Backend skeleton** `backend/` (Cargo workspace): axum HTTP API, config from env.txt/infra/.env, MariaDB via sqlx with migrations (`analyses`, `fields` with a POLYGON column, `messages`, `photos`, `sections`, `reports`, `source_cache`), health endpoint, CORS for the dev frontend.
2. **NASA POWER service:** daily series 1981→today for a point; 40-year normals computed from the daily series; derived indicators: temperature anomaly, heat-stress days, frost days, rain and extremes, FAO-56 reference ET (Penman-Monteith from POWER inputs) and a simple water balance, growing degree days and pest generations for a small pest table (aphid, whitefly, fall armyworm; cite the thresholds used). Cache responses in `source_cache`.
3. **Imagery and terrain:** Planetary Computer STAC search (Sentinel-2 L2A + Landsat C2 L2) for the field polygon, NDVI time series for the polygon (anonymous SAS token), bare-soil periods; NASADEM slope for the polygon.
4. **Earthdata sources:** SMAP L4 surface/root-zone moisture (cloud OPeNDAP, range syntax `[a:b]`), IMERG Late daily rain (GES DISC OPeNDAP, order [time][lon][lat]), GRACE series (reuse `tools/grace_point.py` logic or pre-extracted JSON per example site). Bearer token from env.txt.
5. **Climate projection:** NEX-GDDP-CMIP6 on Planetary Computer. First confirm each model's license on NASA's page; use only open-licensed models; record it in references.
6. **Evidence package + crop/pest catalog:** a JSON catalog (25-40 crops for the Huaral valley: family, role, water need, Kc, temperature thresholds, legume N, pests, beneficial insects and fungi incl. Trichoderma, Chrysoperla, Beauveria) with a citation per field; the analyst stage that turns sources + catalog into a fixed evidence package; a rotation engine that scores at least three strategies (current + 2) on soil, water, pest break, income, risk, with why/evidence/gaps/risks.
7. **Frontend index:** Vite + TypeScript. Three.js globe at sunrise with real NASA textures (Blue Marble, VIIRS night lights) and an atmosphere rim; scroll-driven dive orbit → valley → field; frosted-glass phone showing the app; sections per `index-prompts.md`; EN | ES switch; reduced-motion and low-power fallback to a still frame. The Lemo-Opuscar Satellite Briefing demo engine (`C:/Users/OutWorldDevourer/.claude/plugins/marketplaces/lemolab/styles/satellite-briefing/demo/engine.js`, MIT, © 2026 Lemomo/LemoLab) may be adapted for the globe and dive; keep its copyright notice and cite it. Do not copy its STYLE.md text (CC BY-NC).
8. **App:** field selection (My GPS | Example site; point or drawn polygon with area; place search), workspace with the glass chat (up to 10 photos per message, drag and drop on desktop, camera on phone) and the section column (Your field, Soil, Pest care, Nutrition, Rotation, Technical sheet) with states empty / in progress / ready / with gaps; dashboard tab; same features on phone and desktop.
9. **Consultant:** Claude via the Anthropic Messages API with model `claude-opus-5-5`, vision input for photos, tool use to request the evidence package and to emit structured section updates; system prompt as a friendly agronomy consultant that asks for missing data one or two questions at a time, guides photos, accepts "I don't know" as a gap, and may only cite numbers from the evidence package. If no `anthropic` key is in env.txt, build it fully behind a provider interface and ship a clearly labelled scripted demo mode for the example site; list the key under "Needs the owner".
10. **Export:** technical sheet PDF with all readings, sources, dates and licenses; CSV of the series; share link.
11. **Example sites:** Huaral valley (Peru) fully precomputed and cached so the demo works even if a service is down; one U.S. site if time allows (then add SSURGO/CDL).
12. **Docs:** keep the handbook (EN + ES) current: data sources page, references page with licenses, how it works, AI use. Rebuild with `node handbook/build.mjs`.

## Needs the owner (fill in as you go)

- Anthropic API key in env.txt as `anthropic <key>` (if missing).
- Own crop photos (arverjita) for the example site, in `assets/photos/`.
- Morning: claim the free domain, create the Google Compute Engine e2-micro VM, then deploy; FortiGuard classification; make the repo public before submission; record the 30-second demo.

## Decision log

| When | Decision | Why |
|------|----------|-----|
| 2026-11-14 | Plan written; autonomous run scheduled for 09:00 Peru time | Contest rule: no building before the start |
