//! Axis markers — phantom-typed scalar units.
//!
//! Axes are zero-sized; they exist purely so a `Lattice<BB>` and a `Scalar<PotFraction>` cannot
//! be confused at the type level.

/// Marker for a scalar axis, typically a zero-sized struct like `BB` or
/// `PotFraction`.
pub trait Axis: 'static {}
