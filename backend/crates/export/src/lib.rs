//! Exports for the agronomist: the cited technical sheet (printable HTML → PDF), the CSV of every series, share links.
mod csv;
mod html;
mod routes;
mod sheet;

pub use routes::routes;
