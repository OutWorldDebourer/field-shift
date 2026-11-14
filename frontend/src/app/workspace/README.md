# workspace

- `workspace.ts` — shell (top bar, tabs, backdrop, footer notice, phone bottom sheet) and event wiring (topic-done detection, chime, rail/topic clicks, glossary).
- `chat.ts` — progress rail, chat log, photo tiles with reticle, composer (≤10 full-resolution photos with sizes, drag and drop, camera, voice dictation), crop inputs, closing card.
- `photos.ts` — originals kept unless > 12 MB or > 4096 px (then JPEG quality 0.92, ≤ 4096 px); size text.
- `progress.ts` — guided steps (field → soil → pests → nutrition → rotation → sheet): backend `steps`, or derived from sections; display states; newly-done detection.
- `rail.ts` — chips + "Next topic … I need …" line + chime toggle; the "We have enough information" card.
- `chime.ts` — soft WebAudio chime when a topic is done (mute saved per device; silent with reduced motion).
- `dictation.ts` — voice to text in the box, never sends: Gemini (MediaRecorder → /api/consultant/transcribe, ≤ 120 s) or Browser (Web Speech); Gemini failure falls back to Browser.
- `sections.ts` — section column (states follow the steps; done cards show the step summary), phone round buttons, section bodies, progress rings.
- `dashboard.ts` — dashboard markup, the live map with the honest-scale layer switch, and the season card wiring.
- `forecast.ts`, `forecast-chart.ts` — forecast horizon 3/5/7/10 d (GEOS-FP), outlook 0–6 months (estimate, two scenarios), 2031–2050 projection toggle; composite chart, source lines, day-by-day table.
- `topics.ts` — recommended topics under the alerts (pre-fill the chat, switch to Chat, never send).
- `alerts.ts` — alerts from the evidence (or derived from readings).
- `rotation.ts` — pre-diagnosis card and strategy comparison (practices for perennials).
- `export.ts` — technical sheet (same tab), CSV, download package (ZIP), share link.
- `credit.ts` — Spanish gloss next to the verbatim Copernicus licence sentence.
- `workspace.css`, `guided.css`, `dashboard.css` — styles.
