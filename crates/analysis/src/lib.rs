//! Training result analysis and query interface.
//!
//! This module provides access to stored abstractions, blueprints, and metrics
//! through both a programmatic API and interactive CLI.
//!
//! ## Core Types
//!
//! - [`API`] — PostgreSQL-backed query interface for training artifacts
//! - [`Query`] — Structured query builders for common lookups
//! - [`CLI`] — Interactive command-line interface
//!
//! ## HTTP Handlers
//!
//! The [`handlers`] submodule exposes actix-web routes for the analysis API,
//! supporting neighborhood queries, histogram retrieval, and blueprint lookup.
mod api;
mod cli;
#[cfg(feature = "server")]
pub mod handlers;
mod query;

pub use api::*;
pub use cli::*;
pub use query::*;
