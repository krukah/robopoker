use super::*;
use crate::*;
use rbp_core::*;

/// Game state for Rock-Paper-Scissors, encoded as a node index.
///
/// The game tree has 13 nodes:
/// - Node 0: Root (P1 to act)
/// - Nodes 1-3: P1 chose R/P/S respectively (P2 to act)
/// - Nodes 4-12: Terminal states (P1 chose X, P2 chose Y)
///
/// The `u8` encodes position in this enumeration, enabling O(1)
/// state transitions and payoff lookups.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpsGame(u8);

impl CfrGame for RpsGame {
    type E = RpsEdge;
    type T = RpsTurn;
    /// Creates the root game state (node 0, P1 to act).
    fn root() -> Self {
        Self(0)
    }
    /// Returns whose turn it is based on node index.
    fn turn(&self) -> Self::T {
        match self.0 {
            00..=00 => RpsTurn::P1,
            01..=03 => RpsTurn::P2,
            04..=12 => RpsTurn::Terminal,
            _ => unreachable!(),
        }
    }

    /// Applies an action to transition to the next state.
    /// Uses a lookup table based on current node and action.
    fn apply(&self, edge: Self::E) -> Self {
        match (self.0, edge) {
            (00, RpsEdge::R) => Self(01),
            (00, RpsEdge::P) => Self(02),
            (00, RpsEdge::S) => Self(03),
            (01, RpsEdge::R) => Self(04),
            (01, RpsEdge::P) => Self(05),
            (01, RpsEdge::S) => Self(06),
            (02, RpsEdge::R) => Self(07),
            (02, RpsEdge::P) => Self(08),
            (02, RpsEdge::S) => Self(09),
            (03, RpsEdge::R) => Self(10),
            (03, RpsEdge::P) => Self(11),
            (03, RpsEdge::S) => Self(12),
            _ => unreachable!(),
        }
    }

    /// Computes payoff at terminal nodes.
    ///
    /// Uses `ASYMMETRIC_UTILITY` to weight Scissors outcomes differently,
    /// shifting equilibrium away from uniform 1/3 each.
    fn payoff(&self, turn: Self::T) -> Utility {
        const P_WIN: Utility = 1.;
        const S_WIN: Utility = P_WIN * ASYMMETRIC_UTILITY;
        let direction = match turn {
            RpsTurn::P1 => 0. + 1.,
            RpsTurn::P2 => 0. - 1.,
            _ => unreachable!(),
        };
        let payoff = match self.0 {
            07 => 0. + P_WIN, // P > R
            05 => 0. - P_WIN, // R < P
            06 => 0. + S_WIN, // R > S
            11 => 0. + S_WIN, // S > P
            10 => 0. - S_WIN, // S < R
            09 => 0. - S_WIN, // P < S
            04 | 08 | 12 => 0.0,
            00..=03 => unreachable!("eval at terminal node, depth > 1"),
            _ => unreachable!(),
        };
        direction * payoff
    }
}
