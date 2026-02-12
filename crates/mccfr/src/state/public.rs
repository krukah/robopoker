//! Information set component traits for generic CFR.
//!
//! An information set consists of two components:
//!
//! - **Public** — Observable by all players (e.g., betting history, available actions)
//! - **Private** — Observable only by the acting player (e.g., hole cards)
//!
//! By making [`Information`] generic over both, we achieve:
//! - Different granularities (abstracted vs exact)
//! - Reusable CFR infrastructure across game variants
//! - Clean separation of concerns
//!
//! # Blanket Implementation
//!
//! Any `Information<X, Y>` automatically implements [`TreeInfo`] when
//! `X: Public` — the available actions come from the public state.

use crate::*;

/// A representation of public information in an information set.
///
/// Public information is observable by all players. This includes:
/// - The action history leading to this state
/// - The available actions at this decision point
///
/// # Associated Types
///
/// - `E: TreeEdge` — The action/edge type for this game
/// - `T: TreeTurn` — The turn/player type for this game
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
    fn choices(&self) -> Vec<Self::E>;

    /// Returns the historic paths up to this decision point.
    fn history(&self) -> Vec<Self::E>;
}
