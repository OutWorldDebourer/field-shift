# Field Shift (team InnoHuando)

NASA Space Apps Challenge 2026, challenge "Field Shift: Adapting Farms with NASA Data".
English is the default language everywhere; Spanish is an opt-in switch.

| Path | What it is |
|------|------------|
| `handbook/` | Curated bilingual documentation. Open `handbook/index.html`. |
| `research/` | Raw research notes and live API tests (English) that feed the handbook. |
| `tools/` | One-off data tools (GRACE extraction) and their Python venv (git-ignored). |
| `infra/` | Local Docker services (MariaDB) and deployment files. |
| `Docs/` | Original inputs from the team owner (challenge text, GO Team mail, terms). Do not edit. |
| `design-directions/` | Visual direction comps, prompts and the approved references. |
| `PRODUCT.md` | Product truth for the design skill (users, purpose, constraints). |
| `.impeccable/` | Design skill working files (decision payload, comps, config). |
| `Entrevista/` | Agronomist interview (Ing. Cenia), source for the result sections. |
| `AI_USE.md` | Log of AI assistance, required by the submission rules. |
| `env.txt` | Local secrets. Git-ignored. Never print or commit. |

Rebuild the handbook after editing any page: `node handbook/build.mjs`
