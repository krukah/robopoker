use crate::*;
use kicker::*;

/// A strategy that always checks or calls: enough to drive a rollout to a
/// terminal node without a blueprint.
pub struct Passive;

impl Oracle for Passive {
    fn policy(&self, witness: &Witness) -> Policy {
        let game = witness.head();
        [(if game.may_check() { Edge::Check } else { Edge::Call }, 1.0)]
            .into_iter()
            .collect()
    }
}
