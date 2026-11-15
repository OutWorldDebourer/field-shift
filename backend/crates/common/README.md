# common

| File | What it does |
|------|--------------|
| `src/config.rs` | Reads env.txt and infra/.env; never prints secrets |
| `src/state.rs` | AppState (DB pool, HTTP client, config) |
| `src/error.rs` | ApiError to JSON responses |
| `src/cache.rs` | `get_or_fetch`: source_cache with stale fallback |
| `src/geo.rs` | Field (point or ring), area, bbox, point-in-polygon, WKT |
| `src/usage.rs` | Token usage of every Anthropic call: tracing line and `api_usage` row (input, output, cache read, cache write) |
