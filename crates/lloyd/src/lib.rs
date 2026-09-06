// Without the `server` feature the DB-backed pipeline orchestration (layer
// upload, telemetry) is unreachable, leaving just the clustering primitives;
// this keeps that lean build warning-free.
#![cfg_attr(not(feature = "server"), allow(dead_code))]
//! Hierarchical k-means clustering for strategic abstraction: reduces 3.1
//! trillion poker situations to strategically-equivalent buckets, one street
//! at a time, clustering hands by their distribution over next-street
//! outcomes.
//!
//! River clusters on raw equity; turn, flop, and preflop each cluster on the
//! distribution over the street below. See `README.md` for the pipeline
//! diagram and cost model.
mod abstraction;
mod artifacts;
mod bins;
mod distances;
mod emd;
mod equity;
mod future;
#[cfg(feature = "server")]
mod gap;
mod heuristic;
mod histogram;
mod hyperparams;
mod kmeans;
mod layer;
mod lookup;
#[cfg(feature = "server")]
mod mapping;
mod metric;
mod pair;
mod phi;
mod potential;
#[cfg(feature = "server")]
mod shift;
mod sinkhorn;
mod telemetry;
mod tests;

// Generic Elkan k-means engine (Energy excluded — pokerkit::Energy is the same f32).
pub use elkan::{Absorb, Bounds, Drift, Elkan, Prior, Step};

pub use abstraction::*;
pub use artifacts::*;
pub use bins::*;
pub use distances::*;
pub use emd::*;
pub use equity::*;
pub use future::*;
#[cfg(feature = "server")]
pub use gap::*;
pub use heuristic::*;
pub use histogram::*;
pub use hyperparams::*;
pub use kmeans::*;
pub use layer::*;
pub use lookup::*;
#[cfg(feature = "server")]
pub use mapping::*;
pub use metric::*;
pub use pair::*;
pub use phi::*;
pub use potential::*;
#[cfg(feature = "server")]
pub use shift::*;
pub use sinkhorn::*;
pub use tests::*;
