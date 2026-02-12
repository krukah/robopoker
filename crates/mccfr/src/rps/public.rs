//! RPS public state type alias.
use super::*;

/// RPS public state.
///
/// In RPS, the turn serves as public state since there is
/// no hidden information between decision points.
pub type RpsPublic = RpsTurn;
