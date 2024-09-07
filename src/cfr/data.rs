use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::player::Player;
use crate::Utility;

pub struct Child(pub Data, pub Edge);

impl From<(Data, Edge)> for Child {
    fn from(tuple: (Data, Edge)) -> Self {
        Child(tuple.0, tuple.1)
    }
}
impl From<Child> for (Data, Edge) {
    fn from(child: Child) -> Self {
        (child.0, child.1)
    }
}

pub struct Data(usize); // Either<(Game, Observation)>, Abstraction
/// pot
/// n_bets
/// observation
/// abstraction
/// rotation

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
            _ => unreachable!(),
        }
    }

    pub fn player(&self) -> &Player {
        // todo!("need to calc on the fly or store in struct");
        match self.0 {
            00 => &Player::P1,
            01 | 05 | 09 => &Player::P2,
            02..=04 | 06..=08 | 10..=12 => &Player::Chance,
            _ => unreachable!(),
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
            _ => unreachable!("eval at terminal node, depth > 1"),
        }
    }

    pub fn spawn(&self) -> Vec<Child> {
        // todo!("need to calc on the fly or store in struct");
        match self.0 {
            // P1 moves
            00 => vec![
                Child(Self(01), Edge::RO),
                Child(Self(05), Edge::PA),
                Child(Self(09), Edge::SC),
            ],
            // P2 moves
            01 => vec![
                Child(Self(02), Edge::RO),
                Child(Self(03), Edge::PA),
                Child(Self(04), Edge::SC),
            ],
            05 => vec![
                Child(Self(06), Edge::RO),
                Child(Self(07), Edge::PA),
                Child(Self(08), Edge::SC),
            ],
            09 => vec![
                Child(Self(10), Edge::RO),
                Child(Self(11), Edge::PA),
                Child(Self(12), Edge::SC),
            ],
            // terminal nodes
            02..=04 | 06..=08 | 10..=12 => vec![],
            _ => panic!("invalid node index {}", self.0),
        }
    }
}
