#![allow(dead_code)]

use super::action::Edge;
use super::bucket::Bucket;
use super::player::Player;
use crate::Utility;

pub struct Child {
    pub loca: Local,
    pub edge: Edge,
}
pub struct Local(pub usize);
impl Local {
    pub fn bucket(&self) -> &Bucket {
        match self.0 {
            00 => &Bucket::P1,
            01..=03 => &Bucket::P2,
            04..=12 => &Bucket::Ignore,
            _ => unreachable!(),
        }
    }
    pub fn player(&self) -> &Player {
        match self.0 {
            00 => &Player::P1,
            01..=03 => &Player::P2,
            04..=12 => &Player::Chance,
            _ => unreachable!(),
        }
    }
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
    pub fn children(&self) -> Vec<Child> {
        match self.0 {
            // P1 moves
            00 => vec![
                Child {
                    loca: Self(01),
                    edge: Edge::RO,
                },
                Child {
                    loca: Self(02),
                    edge: Edge::PA,
                },
                Child {
                    loca: Self(03),
                    edge: Edge::SC,
                },
            ],
            // P2 moves
            01 => vec![
                Child {
                    loca: Self(04),
                    edge: Edge::RO,
                },
                Child {
                    loca: Self(05),
                    edge: Edge::PA,
                },
                Child {
                    loca: Self(06),
                    edge: Edge::SC,
                },
            ],
            02 => vec![
                Child {
                    loca: Self(07),
                    edge: Edge::RO,
                },
                Child {
                    loca: Self(08),
                    edge: Edge::PA,
                },
                Child {
                    loca: Self(09),
                    edge: Edge::SC,
                },
            ],
            03 => vec![
                Child {
                    loca: Self(10),
                    edge: Edge::RO,
                },
                Child {
                    loca: Self(11),
                    edge: Edge::PA,
                },
                Child {
                    loca: Self(12),
                    edge: Edge::SC,
                },
            ],
            // terminal nodes
            04..=12 => Vec::new(),
            //
            _ => unreachable!(),
        }
    }
    pub fn root() -> Self {
        Self(0)
    }
}
