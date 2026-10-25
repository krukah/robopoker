//! The one game-specific seam for off-tree action nesting.
use crate::*;
use mccfr::*;

impl<T> OffPayload for T where T: Copy + Eq + Ord + std::hash::Hash + std::fmt::Debug + Send + Sync {}

/// A game whose legal-action set can be extended, at play time, with an **off-abstraction**
/// action — one not in the fixed training grid (an off-tree bet size, for poker).
///
/// This trait is the *entire* domain-specific surface of nesting: the rest of the `nest`
/// wrapper family is generic.
pub trait Augmentable: CfrGame {
    /// The off-abstraction payload (chips, for a betting game).
    type Off: OffPayload;

    /// Apply the off-abstraction action, producing the resulting state.
    ///
    /// Precondition: `off` is a legal action at this node. Legality and any
    /// domain ceilings (e.g. an all-in cap) are the caller's responsibility,
    /// handled upstream — for poker, at translation time.
    fn augment(&self, off: Self::Off) -> Self;
}
