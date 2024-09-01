use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::player::Player;
use crate::Utility;

pub struct Child {
    pub data: Data,
    pub edge: Edge,
}

pub struct Data(usize); // Either<(Game, Observation)>, Abstraction

impl Data {
    pub fn root() -> Self {
        todo!("need to calc on the fly or store in struct");
        Self(0)
    }

    pub fn bucket(&self) -> &Bucket {
        todo!("need to calc on the fly or store in struct");
        match self.0 {
            00 => &Bucket::P1,
            01..=03 => &Bucket::P2,
            04..=12 => &Bucket::Ignore,
            _ => unreachable!(),
        }
    }

    pub fn player(&self) -> &Player {
        todo!("need to calc on the fly or store in struct");
        match self.0 {
            00 => &Player::P1,
            01..=03 => &Player::P2,
            04..=12 => &Player::Chance,
            _ => unreachable!(),
        }
    }

    pub fn stakes(&self) -> Utility {
        todo!("need to calc on the fly or store in struct");
        const HI_STAKES: Utility = 2e0; // we can modify payoffs to verify convergence
        const LO_STAKES: Utility = 1e0;
        match self.0 {
            04 | 08 | 12 => 0.0,
            07 => 0. + LO_STAKES, // P > R
            05 => 0. - LO_STAKES, // R < P
            06 => 0. + HI_STAKES, // R > S
            11 => 0. + HI_STAKES, // S > P
            10 => 0. - HI_STAKES, // S < R
            09 => 0. - HI_STAKES, // P < S
            _ => unreachable!("eval at terminal node, depth > 1"),
        }
    }

    pub fn spawn(&self) -> Vec<Child> {
        todo!("need to calc on the fly or store in struct");
        match self.0 {
            // P1 moves
            00 => vec![
                Child {
                    data: Self(01),
                    edge: Edge::RO,
                },
                Child {
                    data: Self(02),
                    edge: Edge::PA,
                },
                Child {
                    data: Self(03),
                    edge: Edge::SC,
                },
            ],
            // P2 moves
            01 => vec![
                Child {
                    data: Self(04),
                    edge: Edge::RO,
                },
                Child {
                    data: Self(05),
                    edge: Edge::PA,
                },
                Child {
                    data: Self(06),
                    edge: Edge::SC,
                },
            ],
            02 => vec![
                Child {
                    data: Self(07),
                    edge: Edge::RO,
                },
                Child {
                    data: Self(08),
                    edge: Edge::PA,
                },
                Child {
                    data: Self(09),
                    edge: Edge::SC,
                },
            ],
            03 => vec![
                Child {
                    data: Self(10),
                    edge: Edge::RO,
                },
                Child {
                    data: Self(11),
                    edge: Edge::PA,
                },
                Child {
                    data: Self(12),
                    edge: Edge::SC,
                },
            ],
            // terminal nodes
            04..=12 => Vec::new(),
            //
            _ => unreachable!(),
        }
    }
}
