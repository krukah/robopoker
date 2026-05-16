//! Axis markers — phantom-typed scalar units.
//!
//! Axes are zero-sized; they exist purely so a `Lattice<BB>` and a
//! `Scalar<PotFraction>` cannot be confused at the type level.
//!
//! All axes are assumed non-negative — the pseudo-harmonic formula
//! `p = (B-x)(1+A) / (B-A)(1+x)` is only well-defined for `x >= 0`,
//! and every axis the project models (chips, pot fractions, big blinds,
//! bid amounts) satisfies that. Don't declare an [`Axis`] for a
//! signed quantity.

/// Marker for a scalar axis. Implementors are typically zero-sized
/// structs like `BB`, `PotFraction`, `BidAmount`. Values along the
/// axis are assumed non-negative.
pub trait Axis: 'static {}
