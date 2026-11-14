//! Exports for the agronomist: the cited technical sheet (printable HTML → PDF), the CSV of every series, share links.
mod csv;
pub mod glossary;
mod package;
mod html;
mod maps;
mod routes;
mod sheet;

pub use routes::routes;

/// GET /api/glossary: the bilingual glossary used by the app and the sheet.
pub async fn glossary_route() -> axum::Json<Vec<glossary::Term>> { axum::Json(glossary::terms()) }
