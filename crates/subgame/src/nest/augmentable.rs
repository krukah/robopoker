//! The one game-specific seam for off-tree action nesting.
use mccfr::*;

/// The marker bundle an off-tree payload needs to ride through the CFR
/// machinery: copyable, a valid hashmap/btree key, debuggable, thread-safe.
/// For NLHE this is `Chips` (an integer). It is **not** a [`CfrEdge`] — the
/// payload is raw data, not a game action (the action is the enclosing
/// [`NestEdge`](super::NestEdge), which *is* a `CfrEdge`). The blanket impl
/// means every qualifying type satisfies it with nothing to write.
pub trait OffPayload: Copy + Eq + Ord + std::hash::Hash + std::fmt::Debug + Send + Sync {}
impl<T> OffPayload for T where T: Copy + Eq + Ord + std::hash::Hash + std::fmt::Debug + Send + Sync {}

/// A game whose legal-action set can be extended, at play time, with an
/// **off-abstraction** action — one not in the fixed training grid (an
/// off-tree bet size, for poker).
///
/// This trait is the *entire* domain-specific surface of nesting: the rest of
/// the `nest` wrapper family is generic. `depth`/`world` never need anything like it
/// because their synthetic edges are pure meta-actions (a continuation-strategy
/// index, a world tag) that never touch the base game's transition function.
/// An off-tree action is a *real* transition built from raw domain data, so
/// exactly one method — [`Self::augment`] — has to be game-specific.
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
