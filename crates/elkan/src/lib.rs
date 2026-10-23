//! Triangle-inequality accelerated k-means (Elkan, 2003), generic over point and distance.
//! Implement [`Elkan`] (`points`, `centroids`, `distance`, plus [`Absorb`] for centroid updates)
//! and its default methods skip most distance calls via bounds, matching naive k-means exactly.
//! The bounds are pluggable ([`Prune`]): Elkan's per-centroid [`Bounds`] or Hamerly's [`Hamerly`].

mod energy;
pub use energy::*;

mod absorb;
mod bounds;
mod drift;
mod elkan;
mod hamerly;
mod prior;
mod prune;
mod step;

pub use absorb::*;
pub use bounds::*;
pub use drift::*;
pub use elkan::*;
pub use hamerly::*;
pub use prior::*;
pub use prune::*;
pub use step::*;
