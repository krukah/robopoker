//! Database pipeline for training artifacts.
//!
//! Bulk data movement between Rust structures and PostgreSQL, optimized for
//! the large-scale writes required during abstraction and blueprint training.
//!
//! ## Core Types
//!
//! - [`Source`] — Async trait for reading from database
//! - [`Sink`] — Async trait for writing to database
//! - [`Stage`] — Temporary staging table management
//! - [`Check`] — Schema validation and migration status
//! - [`Memory`] — Accumulated values for an info set
//! - [`Record`] — Training progress snapshot for checkpointing
mod check;
mod memory;
mod record;
mod sink;
mod source;
mod stage;

pub use check::*;
pub use memory::*;
pub use record::*;
pub use sink::*;
pub use source::*;
pub use stage::*;
