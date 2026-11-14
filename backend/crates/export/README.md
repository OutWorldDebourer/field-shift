# export

| File | What it does |
|------|--------------|
| `src/sheet.rs` | Technical sheet HTML (A4 print to PDF): readings with source, date, resolution, badge, licence; sections; gaps; notice |
| `src/csv.rs` | CSV of readings and every series |
| `src/html.rs` | Escaping and bilingual helpers |
| `src/routes.rs` | `GET /{id}/sheet`, `GET /{id}/series.csv`, `POST /{id}/share`, `GET /shared/{token}`, `GET /shared/{token}/sheet` |
| `src/glossary.rs` | Bilingual glossary (`GET /api/glossary`) and the sheet annex |
| `src/package.rs` | ZIP for the agronomist: sheet, CSV, full-resolution photos, satellite picture, README |
