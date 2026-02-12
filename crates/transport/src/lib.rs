//! Optimal transport and Earth Mover's Distance computation.
//!
//! This module computes distances between probability distributions over metric
//! spaces, enabling the clustering algorithms to measure strategic similarity
//! between poker situations.
//!
//! ## Algorithms
//!
//! - [`Greenkhorn`] — Sinkhorn-like algorithm with greedy row/column updates
//! - [`Greedy`] — Fast approximate coupling via greedy matching
//!
//! ## Core Types
//!
//! - [`Coupling`] — A transport plan between two distributions
//! - [`Density`] — A discrete probability distribution (histogram)
//! - [`Measure`] — Weighted point mass in the transport problem
//! - [`Support`] — The underlying metric space with pairwise distances
//!
//! ## Usage
//!
//! The Sinkhorn iterations are controlled by temperature, iteration count, and
//! convergence tolerance parameters defined in the crate root. Lower temperature
//! yields sharper transport plans at the cost of numerical stability.
mod coupling;
mod density;
mod greedy;
mod greenkhorn;
mod measure;
mod support;

pub use coupling::*;
pub use density::*;
pub use greedy::*;
pub use greenkhorn::*;
pub use measure::*;
pub use support::*;
