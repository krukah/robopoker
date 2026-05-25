//! Strategic litmus tests for blueprint validation.
//!
//! See `scripts/litmus/README.md` for the human-facing workflow.
//!
//! This crate is the typed core: schema definitions, named-ref resolution,
//! family-matrix expansion, per-kind evaluation, and markdown rendering.
//! It depends only on `rbp-cards` / `rbp-gameplay` for types; the I/O
//! surface is the [`Ops`] trait, implemented by the server crate (or any
//! other backend) so this crate stays cycle-free.

mod api;
mod catalog;
mod compose;
mod evaluate;
mod ops;
mod render;
mod schema;

pub use api::Litmus;
pub use catalog::Catalog;
pub use compose::resolve;
pub use evaluate::{Outcome, Status, evaluate};
pub use ops::Ops;
pub use render::render;
pub use schema::{
    Case, CategoryDef, Direction, Expect, Family, HandDef, Historical, HistoryDef, Scenarios, TestKind, load,
};
