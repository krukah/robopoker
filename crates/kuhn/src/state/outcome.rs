use crate::*;
use pokerkit::*;

/// Terminal outcome of a Kuhn hand.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Outcome {
    Fold(usize),
    Showdown { raised: bool },
}
impl Outcome {
    pub(crate) fn payoff(&self, player: usize, hole: [Card; 2]) -> Utility {
        match self {
            Outcome::Fold(who) => {
                if *who == player {
                    -1.0
                } else {
                    1.0
                }
            }
            Outcome::Showdown { raised } => {
                let stake = if *raised { 2.0 } else { 1.0 };
                match hole[0].rank().cmp(&hole[1].rank()) {
                    std::cmp::Ordering::Greater => {
                        if player == 0 {
                            stake
                        } else {
                            -stake
                        }
                    }
                    std::cmp::Ordering::Less => {
                        if player == 1 {
                            stake
                        } else {
                            -stake
                        }
                    }
                    std::cmp::Ordering::Equal => 0.0,
                }
            }
        }
    }
}
