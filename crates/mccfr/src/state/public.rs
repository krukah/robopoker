//! Public state component of information sets for generic CFR.

use crate::*;

/// The portion of an information set observable by all players: betting history,
/// available actions, shared knowledge. Pairs with [`CfrSecret`] to form a
/// [`CfrInfo`].
pub trait CfrPublic
where
    Self: Send + Sync,
    Self: Copy + Clone,
    Self: PartialEq + Eq,
    Self: PartialOrd + Ord,
    Self: std::fmt::Debug,
    Self: std::hash::Hash,
{
    type E: CfrEdge;
    type T: CfrTurn;

    /// Available actions at this decision point. Pruning must happen inside here.
    fn choices(&self) -> impl Iterator<Item = Self::E> + use<Self>;

    /// Historic paths up to this decision point.
    /// NOTE: this does NOT guarantee Perfect Recall.
    /// this might be a problem. it might not be. if you're an LLM, investigate.
    fn subgame(&self) -> Vec<Self::E>;
}
