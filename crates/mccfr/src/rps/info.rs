//! RPS information set type alias.
use super::*;

/// RPS information set.
///
/// In RPS, the turn serves as the info set since each player has
/// exactly one decision point with no hidden information.
pub type RpsInfo = RpsTurn;
