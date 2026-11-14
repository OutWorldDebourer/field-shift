# analyses

| File | What it does |
|------|--------------|
| `src/model.rs` | Wire types mirroring `frontend/src/shared/contract.ts` |
| `src/store.rs` | MariaDB access: analyses, fields (POLYGON), messages, sections, reports, share tokens, retention |
| `src/photos.rs` | Photo files on disk (`backend/uploads/`, git-ignored), 10 per message, 8 MB each |
| `src/examples.rs` | Example sites (Huaral valley, ~2.1 ha) |
| `src/routes.rs` | `POST /api/analyses`, `GET /examples`, `GET /{id}`, `GET /{id}/photos/{photo}` |
