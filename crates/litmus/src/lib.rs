//! Strategic litmus tests for blueprint validation.
//!
//! This crate is the typed core: schema definitions, named-ref resolution, family-matrix
//! expansion, per-kind evaluation, and markdown rendering.

mod cases;
mod library;
mod pipeline;
mod report;
mod scenarios;
mod schema;
pub use cases::*;
pub use library::*;
pub use pipeline::*;
pub use report::*;
pub use scenarios::*;
pub use schema::*;
