//! Triangle-inequality accelerated k-means clustering (Elkan, 2003).
//!
//! Generic over the point type and the distance function: implement [`Elkan`]
//! for your data (providing `points`, `centroids`, `distance`, and an
//! [`Absorb`] impl for incremental centroid updates) and the trait's default
//! methods run k-means while skipping most distance computations via
//! upper/lower bounds, producing results identical to naive k-means.
//! `step_naive` is kept for verification against the accelerated path.

/// Scalar type for distances and drifts.
pub type Energy = f32;

mod absorb;
mod bounds;
mod drift;
mod elkan;
mod prior;
mod step;

pub use absorb::*;
pub use bounds::*;
pub use drift::*;
pub use elkan::*;
pub use prior::*;
pub use step::*;
