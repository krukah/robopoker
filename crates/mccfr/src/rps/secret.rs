//! RPS private state type alias.
use super::*;

/// RPS private state.
///
/// In RPS, the turn serves as private state since there is
/// no hidden information - each player's situation is symmetric.
pub type RpsSecret = RpsTurn;
