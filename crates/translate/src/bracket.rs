//! Typed bracketing pair for pseudo-harmonic translation.

use crate::*;

/// A pair of bracketing anchors `(lo, hi)` from [`crate::Lattice::bracket`].
///
/// `lo == hi` indicates a clamp at an extreme; `lo != hi` indicates
/// `observed` is strictly between two distinct anchors. Use
/// [`Self::is_clamped`] to discriminate before calling
/// [`crate::Lattice::pharmonic`] (which requires the inside case).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Bracket(Anchor, Anchor);

impl Bracket {
    pub fn new(lo: Anchor, hi: Anchor) -> Self {
        Self(lo, hi)
    }

    pub fn lo(&self) -> Anchor {
        self.0
    }

    pub fn hi(&self) -> Anchor {
        self.1
    }

    /// True when `observed` clamped to a single anchor (extreme of the lattice).
    pub fn is_clamped(&self) -> bool {
        self.0 == self.1
    }
}

/// Construct an inside-the-lattice [`Bracket`] from the upper index `hi`.
/// The pair is `(hi - 1, hi)`. Panics if `hi == 0` (use [`Bracket::new`]
/// with equal anchors for the clamped extremes).
impl From<usize> for Bracket {
    fn from(hi: usize) -> Self {
        if hi >= 1 {
            Self(Anchor::new(hi - 1), Anchor::new(hi))
        } else {
            panic!("Bracket::from(hi) requires hi >= 1; use Bracket::new for clamped extremes")
        }
    }
}
