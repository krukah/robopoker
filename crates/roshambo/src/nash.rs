use crate::*;
use pokerkit::*;

/// This game's unique equilibrium: both players play R 0.4, P 0.4, S 0.2.
pub struct RpsNash;

impl RpsNash {
    pub fn policy(edge: RpsEdge) -> Probability {
        match edge {
            RpsEdge::R => 0.4,
            RpsEdge::P => 0.4,
            RpsEdge::S => 0.2,
        }
    }

    /// The largest gap between `policy` and the equilibrium, over both players and every edge.
    pub fn gap<P>(policy: P) -> Probability
    where
        P: Fn(RpsTurn, RpsEdge) -> Probability,
    {
        [RpsTurn::P1, RpsTurn::P2]
            .into_iter()
            .flat_map(|turn| [RpsEdge::R, RpsEdge::P, RpsEdge::S].map(|edge| (turn, edge)))
            .map(|(turn, edge)| (policy(turn, edge) - Self::policy(edge)).abs())
            .fold(0., Probability::max)
    }
}
