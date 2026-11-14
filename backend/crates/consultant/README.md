# consultant

| File | What it does |
|------|--------------|
| `src/anthropic.rs` | Messages API client (claude-opus-5-5, refusal fallback); key never logged |
| `src/prompt.rs` | Cached system prompt and the per-turn context note |
| `src/tools.rs` | Tools: set_crops, get_evidence, get_advice, update_section, tag_photo, record_gap |
| `src/turn.rs` | One turn: store message and photos, tool loop with vision, store reply |
| `src/demo.rs` | Labelled scripted demo mode (fallback, never reads photos) |
| `src/ratelimit.rs` | 30 live turns/hour per visitor (hashed IP), 400/day global |
| `src/routes.rs` | `POST /api/consultant/{id}/message` (multipart: text, lang, demo, photos) |
| `src/tiles.rs` | Model view of each photo (≤1568 px, under the 5 MB API limit) and native-resolution quarters for tiny insects |
| `src/transcribe.rs` | Voice dictation with Gemini (`POST /api/consultant/transcribe`), transcribe only |
