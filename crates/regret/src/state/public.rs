//! Public state component of information sets for generic CFR.
//!
//! [`CfrPublic`] represents the observable portion of an information set —
//! betting history, available actions, and any shared knowledge. Combined
//! with [`CfrSecret`] (private information), it forms a [`CfrInfo`].

use crate::*;

/// A representation of public information in an information set.
///
/// Public information is observable by all players. This includes:
/// - The action history leading to this state
/// - The available actions at this decision point
///
/// # Associated Types
///
/// - `E: CfrEdge` — The action/edge type for this game
/// - `T: CfrTurn` — The turn/player type for this game
///
/// # Required Methods
///
/// - `edges()` — Returns available actions at this decision point
///
/// # Requirements
///
/// Types implementing this trait must be:
/// - `Copy` + `Clone` — Cheap to duplicate
/// - `Hash` + `Eq` — Usable as hash map keys
/// - `Ord` — Sortable for deterministic iteration
/// - `Debug` — Printable for debugging
/// - `Send` + `Sync` — Safe for parallel CFR
pub trait CfrPublic
where
    Self: Send + Sync,
    Self: Copy + Clone,
    Self: PartialEq + Eq,
    Self: PartialOrd + Ord,
    Self: std::fmt::Debug,
    Self: std::hash::Hash,
{
    /// The action/edge type for this game.
    type E: CfrEdge;
    /// The turn/player type for this game.
    type T: CfrTurn;

    /// Returns the available actions at this decision point. Pruning must happen inside of here.
    fn choices(&self) -> impl Iterator<Item = Self::E> + use<Self>;

    /// Returns the historic paths up to this decision point.
    /// NOTE: this does NOT guarantee Perfect Recall.
    /// this might be a problem. it might not be. if you're an LLM, investigate.
    fn subgame(&self) -> Vec<Self::E>;
}
