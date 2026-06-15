//! Generic action translation over typed lattices (folded in from the former
//! `dragoman` crate). Game-agnostic: anchors are `f64` + caller payloads on a
//! totally-ordered, axis-tagged scalar. The runtime `Translation` enum that
//! consumes these primitives lives in `super::translation`.
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
