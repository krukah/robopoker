//! Hierarchical k-means clustering for strategic abstraction.
//!
//! This module reduces the 3.1 trillion unique poker situations into a tractable
//! number of strategically-equivalent buckets. The abstraction proceeds street-by-street,
//! clustering hands based on their distributions over next-street outcomes.
//!
//! ## Pipeline
//!
//! 1. **River** — Cluster by raw equity (win probability against random hands)
//! 2. **Turn** — Cluster by distribution over river buckets
//! 3. **Flop** — Cluster by distribution over turn buckets
//! 4. **Preflop** — Cluster by distribution over flop buckets
//!
//! ## Core Types
//!
//! - [`Layer`] — A clustering layer mapping observations to abstract buckets
//! - [`Histogram`] — Distribution over child buckets for a given hand
//! - [`Lookup`] — Precomputed observation → bucket mapping
//! - [`Metric`] — Pairwise EMD distances between buckets
//!
//! ## Algorithms
//!
//! - [`Elkan`] — Accelerated k-means with triangle inequality bounds
//! - [`Sinkhorn`] — Entropic optimal transport for EMD computation
//! - [`Absorb`] — Incremental centroid updates during clustering
//!
//! ## Persistence
//!
//! - [`Artifacts`] — Serialization of clustering results to PostgreSQL
//! - [`Distances`] — Precomputed distance matrices for online lookup
mod absorb;
mod artifacts;
mod bins;
mod bounds;
mod distances;
mod elkan;
mod emd;
mod equity;
mod future;
mod heuristic;
mod histogram;
mod layer;
mod lookup;
mod metric;
mod pair;
mod phi;
mod potential;
mod sinkhorn;
mod tests;

pub use absorb::*;
pub use artifacts::*;
pub use bins::*;
pub use bounds::*;
pub use distances::*;
pub use elkan::*;
pub use emd::*;
pub use equity::*;
pub use future::*;
pub use heuristic::*;
pub use histogram::*;
pub use layer::*;
pub use lookup::*;
pub use metric::*;
pub use pair::*;
pub use phi::*;
pub use potential::*;
pub use sinkhorn::*;
pub use tests::*;
