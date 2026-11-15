# crops

Open crop knowledge at `/api/crops`: any crop name → cited record → crop-type-aware advice.

- `src/lib.rs` — routes (`GET /`, `/suggest?q=`, `/resolve?q=`, `POST /advice`) and public `resolve(state, q)`.
- `src/model.rs` — `CropRecord` and traits; every value `{value, unit, citation}` or `{gap}`.
- `src/seed.rs` — verified seed records embedded from `data/*.json`.
- `src/normalize.rs` — EN/ES free text → crop key (accents, plurals, diminutives, typos); type-ahead ranking.
- `src/research.rs` — Claude (claude-opus-5-5 + web search) researches unknown crops; uncited values become gaps. `search_conversation` is the shared web-search loop.
- `src/research_extras.rs` — phase D: weather that favours each pest, critical micronutrients, biostimulant categories for a seed crop, validated against retrieved pages; stored as `data/extras/<key>.json` and applied when the seed loads.
- `src/store.rs` — `crop_traits` cache of AI-researched records (researched once, kept forever).
- `src/advice/mod.rs` — `advice(current, previous, signals)`, `FieldSignals`, `Advice` types.
- `src/advice/rotation.rs` — annual rotation strategies and semi-perennial renewal options, scored 1–5; `proposal` scores the farmer's own idea (perennials as plantations).
- `src/advice/perennial.rs` — perennial practices (inter-row cover, intercropping, residues, biocontrol).
- `src/advice/care.rs` — soil, pest care and nutrition blocks.
- `src/advice/refs.rs` — cited guidance sources.
- `data/` — seed JSON files; `data/README.md` lists every source and its license. `data/extras/` — phase-D research sidecars (AI-researched, verify).
