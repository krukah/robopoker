//! Public state component of information sets for generic CFR.

use crate::*;

/// The portion of an information set observable by all players: betting history, available
/// actions, shared knowledge. Pairs with [`CfrSecret`] to form a [`CfrInfo`]. The public half
/// of an info-set key.
pub trait CfrPublic
where
    Self: Send + Sync,
    Self: Copy + Clone,
    Self: PartialEq + Eq,
    Self: PartialOrd + Ord,
    Self: std::fmt::Debug,
    Self: std::hash::Hash,
{
    type G: CfrGame;

    /// Available actions at this decision point. Pruning must happen inside here.
    ///
    /// This is the "menu is public" property, and it is a well-formedness condition rather than
    /// a convenience: regret and strategy vectors are indexed by action, so every concrete node
    /// sharing an info set must agree on the menu. Two nodes offering different actions would
    /// be distinguishable to the player, and so could not be one info set.
    fn menu(&self) -> impl Iterator<Item = <Self::G as CfrGame>::E> + use<Self>;

    /// How many `menu` items there are. Override when the menu's size is known without building it.
    fn count(&self) -> usize {
        self.menu().count()
    }
}
