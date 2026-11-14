//! The consultant: Claude (Anthropic Messages API, vision + tool use) behind a provider switch, with a clearly
//! labelled scripted demo mode as fallback, and a per-visitor rate limit so the public demo cannot drain the credit.
mod anthropic;
mod demo;
mod prompt;
mod ratelimit;
mod routes;
mod tiles;
mod tools;
mod transcribe;
mod turn;

pub use routes::routes;
pub use turn::{run_turn, NewPhoto, TurnInput};
