//! Hierarchical k-means clustering for strategic abstraction: reduces 3.1 trillion poker
//! situations to strategically-equivalent buckets, one street at a time, clustering hands by
//! their distribution over next-street outcomes.
//!
//! River clusters on raw equity; turn, flop, and preflop each cluster on the distribution over
//! the street below. See `README.md` for the pipeline diagram and cost model.
mod cluster;
mod distance;
mod distribution;
mod hyperparams;
mod output;
mod transport;

// Generic Elkan k-means engine (Energy excluded — pokerkit::Energy is the same f32).
pub use elkan::{Absorb, Bounds, Drift, Elkan, Hamerly, Prior, Prune, Step};

pub use cluster::*;
pub use distance::*;
pub use distribution::*;
pub use hyperparams::*;
pub use output::*;
pub use transport::*;
