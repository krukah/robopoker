use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::player::Player;
use crate::Utility;

/// pot
/// n_bets
/// observation
/// abstraction
/// rotation
pub struct Data(usize);

impl Data {
    pub fn root() -> Self {
        // todo!("need to calc on the fly or store in struct");
        Self(0)
    }

    pub fn bucket(&self) -> &Bucket {
        // todo!("need to calc on the fly or store in struct");
        match self.0 {
            00 => &Bucket::P1,
            01 | 05 | 09 => &Bucket::P2,
            02..=04 | 06..=08 | 10..=12 => &Bucket::IGNORE,
            _ => unreachable!("no other nodes"),
        }
    }

    pub fn player(&self) -> &Player {
        // todo!("need to calc on the fly or store in struct");
        match self.0 {
            00 => &Player::P1,
            01 | 05 | 09 => &Player::P2,
            02..=04 | 06..=08 | 10..=12 => &Player::Chance,
            _ => unreachable!("no other nodes"),
        }
    }

    pub fn stakes(&self) -> Utility {
        // todo!("need to calc on the fly or store in struct");
        const HI_STAKES: Utility = 2e0; // we can modify payoffs to verify convergence
        const LO_STAKES: Utility = 1e0;
        match self.0 {
            02 | 07 | 12 => 0.0,
            03 => 0. - LO_STAKES, // R < P
            04 => 0. + HI_STAKES, // R > S
            06 => 0. + LO_STAKES, // P > R
            08 => 0. - HI_STAKES, // S < R
            10 => 0. - HI_STAKES, // P < S
            11 => 0. + HI_STAKES, // S > P
            _ => unreachable!("evaluate stakes only at terminal nodes"),
        }
    }

    pub fn edges(&self) -> Vec<Edge> {
        // todo!("need to calc on the fly or store in struct");
        match self.0 {
            00 | 01 | 05 | 09 => vec![Edge::RO, Edge::PA, Edge::SC],
            00..=12 => vec![],
            _ => unreachable!("no other nodes"),
        }
    }

    pub fn spawn(&self) -> Vec<(Data, Edge)> {
        // todo!("need to calc on the fly or store in struct");
        match self.0 {
            // P1 moves
            00 => vec![
                (Self(01), Edge::RO),
                (Self(05), Edge::PA),
                (Self(09), Edge::SC),
            ],
            // P2 moves
            01 => vec![
                (Self(02), Edge::RO),
                (Self(03), Edge::PA),
                (Self(04), Edge::SC),
            ],
            05 => vec![
                (Self(06), Edge::RO),
                (Self(07), Edge::PA),
                (Self(08), Edge::SC),
            ],
            09 => vec![
                (Self(10), Edge::RO),
                (Self(11), Edge::PA),
                (Self(12), Edge::SC),
            ],
            // terminal nodes
            02..=04 | 06..=08 | 10..=12 => vec![],
            _ => unreachable!("no other nodes"),
        }
    }
}
