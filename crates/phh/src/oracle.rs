//! The one question this crate asks a strategy, and the shape of an answer.
//!
//! [`Oracle`] is the smallest surface both consumers need. Tier 2
//! ([`crate::Agreement`]) asks once per logged decision; Tier 3
//! ([`crate::Ledger`]) asks thousands of times inside a rollout. Anything that
//! turns a [`Witness`] into edge weights satisfies it — `parlor::Brain::distrib`
//! does, so every cell of the bot-config hypercube is scoreable, and so does an
//! HTTP client against a running backend.
//!
//! The trait lives here rather than being imported from `parlor` so that `phh`
//! stays free of the blueprint, the subgame solver and the database. The binary
//! owns the impls; the crate owns the question.

use kicker::*;
use pokerkit::*;
use rand::Rng;

/// Edge weights at one information set.
///
/// Weights are kept exactly as the strategy handed them over — the HTTP API
/// returns unnormalized accumulated regret-matching weights, an in-memory
/// profile returns normalized ones — and normalized on read, so no caller has to
/// remember which it is holding. Non-finite and non-positive weights are ignored
/// throughout.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Policy(Vec<(Edge, Probability)>);

impl Policy {
    /// Every edge carrying usable mass, in the order the strategy offered them.
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.0
            .iter()
            .filter(|(_, w)| w.is_finite() && *w > 0.0)
            .map(|(edge, _)| *edge)
    }

    /// How many edges carry a weight, sound or not.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True when the strategy had nothing to say — an unreached information set.
    pub fn is_empty(&self) -> bool {
        self.total() <= 0.0
    }

    /// Sum of the usable weights: the normalizer for [`mass`](Self::mass).
    pub fn total(&self) -> Probability {
        self.0
            .iter()
            .map(|(_, w)| *w)
            .filter(|w| w.is_finite() && *w > 0.0)
            .sum()
    }

    /// Normalized weight on `edge`; zero when absent or when nothing has mass.
    pub fn mass(&self, edge: Edge) -> Probability {
        match self.total() {
            total if total > 0.0 => self.0.iter().find(|(e, _)| *e == edge).map_or(0.0, |(_, w)| w / total),
            _ => 0.0,
        }
    }

    /// The heaviest edge — what the strategy would do if it never mixed.
    pub fn favorite(&self) -> Option<Edge> {
        self.0
            .iter()
            .filter(|(_, w)| w.is_finite() && *w > 0.0)
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(edge, _)| *edge)
    }

    /// One edge drawn in proportion to its weight — what the strategy actually does.
    ///
    /// [`favorite`](Self::favorite) overstates a mixed strategy by playing its top
    /// action purely, so a rollout draws rather than taking the argmax.
    pub fn draw(&self, rng: &mut impl Rng) -> Option<Edge> {
        let mut cursor = match self.total() {
            total if total > 0.0 => rng.random_range(0.0..total),
            _ => return None,
        };
        self.0
            .iter()
            .filter(|(_, w)| w.is_finite() && *w > 0.0)
            .find(|(_, w)| {
                cursor -= *w;
                cursor <= 0.0
            })
            .map(|(edge, _)| *edge)
            .or_else(|| self.favorite())
    }
}

impl FromIterator<(Edge, Probability)> for Policy {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Edge, Probability)>,
    {
        Self(iter.into_iter().collect())
    }
}

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
    use rand::SeedableRng;

    fn policy() -> Policy {
        [(Edge::Fold, 1.0), (Edge::Check, 3.0)].into_iter().collect()
    }

    fn rng(seed: u64) -> rand::rngs::SmallRng {
        rand::rngs::SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn weights_normalize_on_read() {
        assert_eq!(policy().mass(Edge::Fold), 0.25);
        assert_eq!(policy().mass(Edge::Check), 0.75);
        assert_eq!(policy().mass(Edge::Shove), 0.0);
    }

    #[test]
    fn an_unreached_information_set_is_empty() {
        assert!(Policy::default().is_empty());
        assert_eq!(Policy::default().favorite(), None);
        assert_eq!(Policy::default().draw(&mut rng(0)), None);
    }

    /// Drawing has to track the weights, not the argmax — a rollout that always
    /// plays the favorite is measuring a bot nobody deployed.
    #[test]
    fn drawing_tracks_the_weights() {
        let ref mut rng = rng(42);
        let folds = (0..4000)
            .filter_map(|_| policy().draw(rng))
            .filter(|e| *e == Edge::Fold)
            .count();
        assert_eq!(policy().favorite(), Some(Edge::Check));
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
        assert_eq!(policy.draw(&mut rng(0)), Some(Edge::Shove));
    }
}
