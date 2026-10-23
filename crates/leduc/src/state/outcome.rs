use crate::*;
use pokerkit::*;

/// Terminal outcome of a Leduc hand.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Outcome {
    FoldR1(usize),
    FoldR2(Card, Spot, usize),
    Showdown(Card, Spot, Spot),
}
impl Outcome {
    fn pot(&self) -> [u8; 2] {
        match self {
            Outcome::FoldR1(who) => {
                let mut p = [1, 1];
                p[1 - who] += 2;
                p
            }
            Outcome::FoldR2(_, r1, who) => {
                let base = if r1.raised() { 3 } else { 1 };
                let mut p = [base, base];
                p[1 - who] += 4;
                p
            }
            Outcome::Showdown(_, r1, r2) => {
                let base = if r1.raised() { 3 } else { 1 };
                let extra = if r2.raised() { 4 } else { 0 };
                [base + extra, base + extra]
            }
        }
    }

    pub(crate) fn payoff(&self, p: usize, hole: [Card; 2]) -> Utility {
        let pot = self.pot();
        match self {
            Outcome::FoldR1(who) | Outcome::FoldR2(_, _, who) => {
                if *who == p {
                    -(pot[p] as Utility)
                } else {
                    pot[*who] as Utility
                }
            }
            Outcome::Showdown(board, _, _) => {
                let rank = board.rank();
                let r0 = hole[0].rank();
                let r1 = hole[1].rank();
                let pair0 = r0 == rank;
                let pair1 = r1 == rank;
                let winner = match (pair0, pair1) {
                    (true, false) => Some(0usize),
                    (false, true) => Some(1usize),
                    _ => match r0.cmp(&r1) {
                        std::cmp::Ordering::Greater => Some(0),
                        std::cmp::Ordering::Less => Some(1),
                        std::cmp::Ordering::Equal => None,
                    },
                };
                match winner {
                    None => 0.0,
                    Some(w) if w == p => pot[1 - p] as Utility,
                    Some(_) => -(pot[p] as Utility),
                }
            }
        }
    }
}
