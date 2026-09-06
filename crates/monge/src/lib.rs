//! Optimal transport and Earth Mover's Distance: distances between
//! probability distributions over metric spaces, which is how clustering
//! measures strategic similarity between poker situations.
//!
//! Sinkhorn iteration is driven by temperature, iteration count, and
//! convergence tolerance; lower temperature sharpens the transport plan at the
//! cost of numerical stability.
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
