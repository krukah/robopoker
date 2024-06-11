use super::action::Edge;
use super::bucket::Bucket;
use super::player::Player;
use crate::Utility;

pub(crate) struct Local(usize);

impl Local {
    /// root of tree is a static method
    pub fn root() -> Self {
        Self(0)
    }
    /// abstraction
    pub fn bucket(&self) -> &Bucket {
        match self.0 {
            00 => &Bucket::P1,
            01..=03 => &Bucket::P2,
            04..=12 => &Bucket::Ignore,
            _ => unreachable!(),
        }
    }
    /// attribution
    pub fn player(&self) -> &Player {
        match self.0 {
            00 => &Player::P1,
            01..=03 => &Player::P2,
            04..=12 => &Player::Chance,
            _ => unreachable!(),
        }
    }
    /// local tree generation
    pub fn spawn(&self) -> Vec<(Self, Edge)> {
        match self.0 {
            // P1 moves
            00 => vec![
                (Self(01), Edge::RK),
                (Self(02), Edge::PA),
                (Self(03), Edge::SC),
            ],
            // P2 moves
            01 => vec![
                (Self(04), Edge::RK),
                (Self(05), Edge::PA),
                (Self(06), Edge::SC),
            ],
            02 => vec![
                (Self(07), Edge::RK),
                (Self(08), Edge::PA),
                (Self(09), Edge::SC),
            ],
            03 => vec![
                (Self(10), Edge::RK),
                (Self(11), Edge::PA),
                (Self(12), Edge::SC),
            ],
            // terminal nodes
            04..=12 => Vec::new(),
            //
            _ => unreachable!(),
        }
    }
    /// utility to player
    pub fn payoff(&self, player: &Player) -> Utility {
        const HI_STAKES: Utility = 2e0; // we can modify payoffs to verify convergence
        const LO_STAKES: Utility = 1e0;
        let direction = match player {
            Player::P1 => 0. + 1.,
            Player::P2 => 0. - 1.,
            _ => unreachable!("payoff should not be queried for chance"),
        };
        let payoff = match self.0 {
            04 | 08 | 12 => 0.0,
            07 => 0. + LO_STAKES, // P > R
            05 => 0. - LO_STAKES, // R < P
            06 => 0. + HI_STAKES, // R > S
            11 => 0. + HI_STAKES, // S > P
            10 => 0. - HI_STAKES, // S < R
            09 => 0. - HI_STAKES, // P < S
            _ => unreachable!("eval at terminal node, depth > 1"),
        };
        direction * payoff
    }
}
