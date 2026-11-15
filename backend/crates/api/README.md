# api

| File | What it does |
|------|--------------|
| `src/main.rs` | Starts the server, runs migrations, mounts routers, CORS for localhost dev |
| `src/health.rs` | `GET /api/health`: DB status and which keys are configured |
| `src/bin/crop_extras.rs` | `cargo run --release -p api --bin crop_extras pea …` (paid): phase-D research of seed crops into `crops/data/extras/` |
| `src/bin/token_probe.rs` | `cargo run --release -p api --bin token_probe [en\|es]`: input tokens per tool-loop request of one scripted Huaral turn (free `count_tokens`, no paid call) |
