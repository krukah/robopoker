//! Axis-typed raise grid for action translation.

use fulcrum::*;

/// Axis-typed raise grid. The variant tag is the axis discriminant —
/// callers dispatch on it to choose the right `Lattice` axis
/// without a magic `opening: bool`.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Grid {
    /// BB-relative open sizes (preflop depth=0 under Pluribus).
    Opening(&'static [Chips]),
    /// Indices into `RAISES` for pot-fraction raise sizes. Callers
    /// resolve to `(numer, denom)` via `RAISES[i]` on each iteration.
    /// This collapses the per-cell typed constants — everything
    /// dispatches off the single `PLURIBUS_INDICES` source-of-truth.
    Postflop(&'static [usize]),
}
