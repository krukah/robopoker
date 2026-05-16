//! Axis-typed raise grid for action translation.

use rbp_core::*;

/// Axis-typed raise grid. The variant tag is the axis discriminant —
/// callers dispatch on it to choose the right [`crate::Lattice`] axis
/// without a magic `opening: bool`.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Grid {
    /// BB-relative open sizes (preflop depth-0 under Pluribus).
    Opening(&'static [Chips]),
    /// Pot-fraction raise sizes `(numer, denom)` for everything else.
    Postflop(&'static [(Chips, Chips)]),
}
