//! Triangle-inequality accelerated k-means clustering (Elkan, 2003).
//!
//! Generic over the point type and the distance function: implement [`Elkan`]
//! for your data (providing `points`, `centroids`, `distance`, and an
//! [`Absorb`] impl for incremental centroid updates) and the trait's default
//! methods run k-means while skipping most distance computations via
//! upper/lower bounds, producing results identical to naive k-means.
//!
//! - [`Elkan`] — the accelerated algorithm (with `step_naive` for verification)
//! - [`Absorb`] — incremental centroid aggregation for the point type
//! - [`Bounds`] — per-point distance bounds maintained across iterations
//! - [`Drift`] — per-centroid movement between iterations
//! - [`Step`] / [`Prior`] — iteration bookkeeping helpers

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
