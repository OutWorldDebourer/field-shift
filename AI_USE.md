# AI use log

The Space Apps rules require stating where and how AI ("SI") tools were used. This log feeds the "Use of Super Intelligence (SI)" field of the project page. AI-assisted code and data are also marked in file headers or commit messages.

| Date | Tool | Used for | Output in the repository |
|------|------|----------|--------------------------|
| 2026-11-14 | Claude (Anthropic), via Claude Code | Reading the official challenge page, guides and forum; researching NASA data access; drafting documentation in English and Spanish | `handbook/`, `research/`, `README.md` |
| 2026-11-14 | Gemini text-to-speech (Google AI Studio), voice Algenib | Narration samples for the demo films (English and Spanish) | `assets/voice/` |
| 2026-11-14 | OpenAI gpt-image-2 (via the impeccable skill's `generate-image.mjs`, one call) | Attempted plate of two pea-plant photo tiles for the index hero phone; the result was a flat grey frame and was discarded, nothing shipped | none (deleted) |
| 2026-11-14 | Crops (ffmpeg) of the owner-approved AI design frames `design-directions/approved-orbit-shot1.webp` and `approved-index-hero.webp` | Illustrative orbit still under the index (shown only if the live globe does not render) and two example pea photos in the hero phone and demo chat; shown with the visible "SI-assisted visuals" tag; not satellite evidence | `frontend/public/img/orbit-still.jpg`, `pea-plant.jpg`, `pea-leaf.jpg` |
| 2026-11-14 | Claude (Anthropic), via Claude Code | Frontend code (index, app, styles, EN/ES strings) | `frontend/src/index/`, `frontend/src/app/`, `frontend/src/shared/` |
| 2026-11-14 | Claude (Anthropic), via Claude Code (Opus 5.5 coordinating; subagents on Opus 5.5 and Claude Fable 5.2 for the globe) | Rust backend (services for POWER, imagery, Earthdata, climate, crops, evidence, consultant, export), Three.js globe, research of crop traits and pest thresholds with citations, design audits by independent auditor agents | `backend/`, `frontend/src/globe/`, `backend/crates/crops/data/`, `design-directions/audit-log.md` |
| 2026-11-14 | Claude `claude-opus-5-5` via the Anthropic API, at runtime | The in-app consultant (reads the farmer's text and photos, may only cite numbers from the evidence package) and the research of trait cards for crops outside the seed base (labelled "AI-researched, verify") | Generated at runtime; disclosed in the UI and on the technical sheet |
| 2026-11-14 | Claude (Anthropic), via Claude Code, with the Lemo-Opuscar skill | Demo films: composition, cursor and reticle choreography, subtitles, sound design and mix, built from frames of the real globe and real app screenshots (no generated imagery); narration is the Gemini Algenib takes above | `film/` |

Not AI-generated: the project idea and the choice of how to solve the challenge belong to the team.
