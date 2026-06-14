//! Server-side litmus glue.
//!
//! Provides a concrete `litmus::Ops` impl backed by `StrategyAPI` /
//! `TrainingAPI`, plus actix-web handlers that delegate to `litmus::Litmus`.
//! Means external callers can run the litmus catalog over HTTP without
//! needing direct DB access.

mod backend;
pub mod handlers;

pub use backend::Backend;
