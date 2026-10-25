//! The one question this crate asks a strategy, and the shape of an answer.
//!
//! [`Oracle`] is the smallest surface both consumers need. Tier 2 ([`crate::Agreement`]) asks
//! once per logged decision; Tier 3 ([`crate::Ledger`]) asks thousands of times inside a
//! rollout.

use super::*;
use kicker::*;

/// A strategy that can be asked what it would do at an information set.
///
/// One method, deliberately synchronous: a rollout makes thousands of these
/// calls in sequence, so anything with a round trip in it is the wrong shape.
/// Point it at an in-memory blueprint.
pub trait Oracle {
    /// Edge weights at `witness`. An empty [`Policy`] means the strategy has
    /// never been to this information set — callers decide what that costs.
    fn policy(&self, witness: &Witness) -> Policy;
}

impl<O> Oracle for &O
where
    O: Oracle + ?Sized,
{
    fn policy(&self, witness: &Witness) -> Policy {
        (**self).policy(witness)
    }
}

impl<O> Oracle for Box<O>
where
    O: Oracle + ?Sized,
{
    fn policy(&self, witness: &Witness) -> Policy {
        (**self).policy(witness)
    }
}

impl<O> Oracle for std::sync::Arc<O>
where
    O: Oracle + ?Sized,
{
    fn policy(&self, witness: &Witness) -> Policy {
        (**self).policy(witness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pokerkit::Probability;
    use rand::SeedableRng;

    impl Policy {
        fn lopsided() -> Self {
            [(Edge::Fold, 1.0), (Edge::Check, 3.0)].into_iter().collect()
        }

        fn rng(seed: u64) -> rand::rngs::SmallRng {
            rand::rngs::SmallRng::seed_from_u64(seed)
        }
    }

    #[test]
    fn weights_normalize_on_read() {
        assert_eq!(Policy::lopsided().mass(Edge::Fold), 0.25);
        assert_eq!(Policy::lopsided().mass(Edge::Check), 0.75);
        assert_eq!(Policy::lopsided().mass(Edge::Shove), 0.0);
    }

    #[test]
    fn an_unreached_information_set_is_empty() {
        assert!(Policy::default().is_empty());
        assert_eq!(Policy::default().favorite(), None);
        assert_eq!(Policy::default().draw(&mut Policy::rng(0)), None);
    }

    /// Drawing has to track the weights, not the argmax — a rollout that always
    /// plays the favorite is measuring a bot nobody deployed.
    #[test]
    fn drawing_tracks_the_weights() {
        let ref mut rng = Policy::rng(42);
        let folds = (0..4000)
            .filter_map(|_| Policy::lopsided().draw(rng))
            .filter(|e| *e == Edge::Fold)
            .count();
        assert_eq!(Policy::lopsided().favorite(), Some(Edge::Check));
        assert!((900..1100).contains(&folds), "drew Fold {folds} times in 4000, want ~1000");
    }

    #[test]
    fn unsound_weights_are_ignored() {
        let policy = [(Edge::Fold, Probability::NAN), (Edge::Check, -1.0), (Edge::Shove, 2.0)]
            .into_iter()
            .collect::<Policy>();
        assert_eq!(policy.total(), 2.0);
        assert_eq!(policy.mass(Edge::Shove), 1.0);
        assert_eq!(policy.favorite(), Some(Edge::Shove));
        assert_eq!(policy.draw(&mut Policy::rng(0)), Some(Edge::Shove));
    }
}
