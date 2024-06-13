#![allow(dead_code)]

pub(crate) struct Local(pub usize);
use super::action::Edge;
use super::bucket::Bucket;
use super::player::Player;
use crate::Utility;
impl Local {
    pub fn root() -> Self {
        Self(0)
    }
    pub fn spawn(&self) -> Vec<(Self, Edge)> {
        match self.0 {
            // P1 moves
            00 => vec![
                (Local(01), Edge::RK),
                (Local(02), Edge::PA),
                (Local(03), Edge::SC),
            ],
            // P2 moves
            01 => vec![
                (Local(04), Edge::RK),
                (Local(05), Edge::PA),
                (Local(06), Edge::SC),
            ],
            02 => vec![
                (Local(07), Edge::RK),
                (Local(08), Edge::PA),
                (Local(09), Edge::SC),
            ],
            03 => vec![
                (Local(10), Edge::RK),
                (Local(11), Edge::PA),
                (Local(12), Edge::SC),
            ],
            // terminal nodes
            04..=12 => Vec::new(),
            //
            _ => unreachable!(),
        }
    }
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
}

pub(crate) struct KLocal(pub usize);
impl KLocal {
    pub fn bucket(&self) -> &KBucket {
        match self.0 {
            01..=03 => &KBucket::K1,
            04..=06 => &KBucket::Q1,
            07..=09 => &KBucket::J1,
            0 => unreachable!(),
            _ => unreachable!(),
        };
        todo!("map integer representation to bucket");
    }
    pub fn player(&self) -> &KPlayer {
        match self.0 {
            0 => &KPlayer::Dealer,
            01..=03 | 07..=09 => &KPlayer::P1,
            04..=06 | 10..=12 => &KPlayer::P2,
            _ => unreachable!(),
        };
        todo!("map integer representation to player");
    }
    pub fn payoff(&self, _: &Player) -> Utility {
        todo!("map integer representation to utility");
    }
}
pub(crate) enum KBucket {
    K1,
    K2,
    Q1,
    Q2,
    J1,
    J2,
}
pub(crate) enum KPlayer {
    Dealer,
    P1,
    P2,
}
