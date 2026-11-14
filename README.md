# Field Shift (team InnoHuando)

NASA Space Apps Challenge 2026, challenge "Field Shift: Adapting Farms with NASA Data".
English is the default language everywhere; Spanish is an opt-in switch.

| Path | What it is |
|------|------------|
| `backend/` | Rust API (axum + MariaDB): NASA data services, evidence package, crop knowledge, consultant, export. Serves the built app. |
| `frontend/` | TypeScript + Vite web app: index with the Three.js globe, and the app (field, chat, sections, dashboard, rotation). EN \| ES. |
| `film/` | Demo films (English submitted, Spanish for the owner), treatment, narration. |
| `handbook/` | Curated bilingual documentation. Open `handbook/index.html`. |
| `research/` | Raw research notes and live API tests (English) that feed the handbook. |
| `assets/` | Project media (voice samples, own crop photos). |
| `tools/` | One-off tools (GRACE extraction, Gemini narration) and their Python venv (git-ignored). |
| `infra/` | Local Docker services (MariaDB) and deployment files. |
| `Docs/` | Original inputs from the team owner (challenge text, GO Team mail, terms). Do not edit. |
| `design-directions/` | Visual direction comps, prompts and the approved references. |
| `PRODUCT.md` | Product truth for the design skill (users, purpose, constraints). |
| `.impeccable/` | Design skill working files (decision payload, comps, config). |
| `Entrevista/` | Agronomist interview (Ing. Cenia), source for the result sections. |
| `BUILD_PLAN.md` | Milestones, rules and decision log for the build (autonomous runs read this first). |
| `AI_USE.md` | Log of AI assistance, required by the submission rules. |
| `env.txt` | Local secrets. Git-ignored. Never print or commit. |

Run locally: `docker compose -f infra/compose.yaml --env-file infra/.env up -d`, then `cd frontend && npm install && npx vite build`, then `cd backend && cargo run --release --bin fieldshift`, and open http://127.0.0.1:8787. Details: `handbook/en/06-how-it-works.md`.

Rebuild the handbook after editing any page: `node handbook/build.mjs`
