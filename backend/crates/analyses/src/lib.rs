//! Analyses storage: an analysis = one field + crops + conversation + photos + result sections + evidence reports.
pub mod examples;
pub mod model;
pub mod photos;
pub mod place;
mod routes;
pub mod steps;
pub mod store;

pub use model::*;
pub use routes::routes;
