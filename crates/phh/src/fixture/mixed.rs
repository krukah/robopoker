use crate::*;
use kicker::*;

/// A strategy that genuinely mixes, so its continuation depends on the random stream.
pub struct Mixed;

impl Oracle for Mixed {
    fn policy(&self, witness: &Witness) -> Policy {
        let game = witness.head();
        [
            (if game.may_check() { Edge::Check } else { Edge::Call }, 2.0),
            (if game.may_check() { Edge::Raise(Odds::new(1, 2)) } else { Edge::Fold }, 1.0),
        ]
        .into_iter()
        .collect()
    }
}
