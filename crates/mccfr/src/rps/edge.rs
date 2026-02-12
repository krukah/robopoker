use crate::*;
use rbp_transport::*;

/// Actions in Rock-Paper-Scissors.
///
/// Each player can choose Rock, Paper, or Scissors.
/// Standard RPS rules apply: R beats S, S beats P, P beats R.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RpsEdge {
    /// Rock — beats Scissors, loses to Paper.
    R,
    /// Paper — beats Rock, loses to Scissors.
    P,
    /// Scissors — beats Paper, loses to Rock.
    S,
}

impl Support for RpsEdge {}
impl CfrEdge for RpsEdge {}
