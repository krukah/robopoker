//! Generic action translation over typed lattices on a totally-ordered scalar axis.
//!
//! Game-agnostic: anchors are plain `f64` paired with caller-defined payloads
//! `P` (defaulting to `()`) and tagged with a phantom [`Axis`] marker. The same
//! machinery serves poker bet sizing, auction bidding, continuous-time stopping,
//! resource allocation, or any extensive-form game with continuous actions
//! abstracted onto a finite discrete grid.
//!
//! ## Public surface
//!
//! - [`Lattice<A, P>`] — central type; owns `(scalar, payload)` pairs.
//! - [`Scalar<A>`] — finite scalar tagged with axis (the observation type).
//! - [`Anchor`] — opaque lattice index.
//! - [`Bracket`] — pair of bracketing anchors from [`Lattice::bracket`].
//! - [`Translated<P, F>`] — resolution result: `Snap(P) | Free(F)`.
//! - [`Axis`] — axis marker (assumed non-negative).
//!
//! The runtime-dispatched `Translation` enum that consumes these primitives
//! lives in `pokerkit` next to `Regime` — it's project configuration,
//! not a library primitive.

mod anchor;
mod axis;
mod bracket;
mod lattice;
mod scalar;
mod translated;

pub use anchor::*;
pub use axis::*;
pub use bracket::*;
pub use lattice::*;
pub use scalar::*;
pub use translated::*;
