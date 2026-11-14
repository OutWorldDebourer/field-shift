# backend

Rust API (axum) + MariaDB (sqlx). Run from here: `cargo run --bin fieldshift` (needs MariaDB from `infra/`). Listens on 127.0.0.1:8787, routes under `/api`.

| Path | What it is |
|------|------------|
| `Cargo.toml` | Workspace: one crate per business area |
| `migrations/` | SQL schema, applied at startup |
| `crates/` | `api` (binary), `common` (config, state, cache, geometry) and one crate per service |
