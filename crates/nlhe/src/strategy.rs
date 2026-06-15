use super::*;
use kicker::*;
use mccfr::*;
use monge::*;
use pokerkit::*;
use std::collections::BTreeMap;

/// A trained strategy for a specific information set.
///
/// Stores accumulated probability mass per action, which can be normalized
/// to produce the actual mixed strategy. This is the output format for
/// querying trained blueprint strategies.
///
/// # Normalization
///
/// Raw accumulated values are not probabilities. Call [`policy()`](Self::policy)
/// to get a normalized distribution that sums to 1. A minimum probability
/// floor (`POLICY_MIN`) prevents zero-probability actions.
///
/// # Density Implementation
///
/// Implements [`Density`] for sampling and probability queries, using the
/// same normalization logic as `policy()`.
#[derive(Debug, Clone, PartialEq)]
pub struct Strategy {
    info: NlheInfo,
    accumulated: BTreeMap<Edge, Probability>,
    visits: BTreeMap<Edge, u32>,
    payoff: f32,
}

impl Strategy {
    /// The information set this strategy applies to.
    pub fn info(&self) -> &NlheInfo {
        &self.info
    }
    /// Raw accumulated probability mass per action.
    pub fn accumulated(&self) -> &BTreeMap<Edge, Probability> {
        &self.accumulated
    }
    /// Encounter visits per action.
    pub fn visits(&self) -> &BTreeMap<Edge, u32> {
        &self.visits
    }
    /// Expected value of this information set.
    /// Stored as an incremental mean, directly readable.
    pub fn payoff(&self) -> f32 {
        self.payoff
    }
    /// Normalized action probabilities (sums to 1).
    /// Applies minimum probability floor to prevent zero weights.
    pub fn policy(&self) -> BTreeMap<Edge, Probability> {
        let denom = self.accumulated.values().map(|&p| p.max(EPSILON)).sum::<Probability>();
        self.accumulated
            .iter()
            .map(|(&edge, &policy)| (edge, policy.max(EPSILON) / denom))
            .collect()
    }
    /// Argmax (Dirac-sharpened) copy: all probability mass collapses
    /// onto the highest-mass edge, others zero. Pure post-processing
    /// on the policy distribution — visits and payoff describe the
    /// underlying training and stay untouched.
    pub fn argmax(&self) -> Self {
        Self {
            info: self.info,
            accumulated: argmax(&self.accumulated),
            visits: self.visits.clone(),
            payoff: self.payoff,
        }
    }
}

/// Collapses a probability distribution to a Dirac delta on its mode:
/// the max-mass key gets 1.0, all others 0.0. Ties broken by
/// `BTreeMap` ordering. Reused by [`Strategy::argmax`] and the
/// `Dirac` brain in `parlor` so they share semantics.
pub fn argmax<K>(probs: &BTreeMap<K, Probability>) -> BTreeMap<K, Probability>
where
    K: Ord + Copy,
{
    let mode = probs
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(k, _)| *k);
    probs
        .keys()
        .map(|k| (*k, if Some(*k) == mode { 1.0 } else { 0.0 }))
        .collect()
}

impl Density for Strategy {
    type Support = NlheEdge;

    fn density(&self, edge: &Self::Support) -> Probability {
        let denom = self.accumulated.values().map(|&p| p.max(EPSILON)).sum::<Probability>();
        self.accumulated
            .get(edge.as_ref())
            .map_or(0., |&p| p.max(EPSILON) / denom)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.accumulated.keys().copied().map(NlheEdge::from)
    }
}

impl From<(NlheInfo, Vec<Decision<NlheEdge>>)> for Strategy {
    fn from((info, decisions): (NlheInfo, Vec<Decision<NlheEdge>>)) -> Self {
        let payoff = decisions.first().map_or(0.0, |d| d.payoff);
        let accumulated = info
            .choices()
            .into_iter()
            .map(|edge| {
                (
                    edge,
                    decisions
                        .iter()
                        .find(|d| Edge::from(d.edge) == edge)
                        .map(|d| d.mass)
                        .expect("empty decision tree"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let visits = info
            .choices()
            .into_iter()
            .map(|edge| {
                (
                    edge,
                    decisions
                        .iter()
                        .find(|d| Edge::from(d.edge) == edge)
                        .map(|d| d.visits)
                        .expect("empty decision tree"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        Self {
            info,
            accumulated,
            visits,
            payoff,
        }
    }
}

impl From<ApiStrategy> for Strategy {
    fn from(api: ApiStrategy) -> Self {
        let info = NlheInfo::from((api.history, api.present, api.choices));
        Self {
            info,
            accumulated: api.accumulated,
            visits: api.visits,
            payoff: api.payoff,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pokerkit::Arbitrary;

    fn build(data: &[(Action, Probability)]) -> Strategy {
        let accumulated = data.iter().map(|(a, p)| (Edge::from(*a), *p)).collect();
        let visits = data.iter().map(|(a, _)| (Edge::from(*a), 0u32)).collect();
        let info = NlheInfo::random();
        Strategy {
            info,
            accumulated,
            visits,
            payoff: 0.0,
        }
    }

    fn sums(s: &Strategy) -> Probability {
        s.policy().values().sum()
    }

    fn close(a: Probability, b: Probability) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn unitarity() {
        let s = build(&[(Action::Fold, 10.0), (Action::Check, 20.0), (Action::Call(10), 30.0)]);
        assert!(close(sums(&s), 1.0));
    }

    #[test]
    fn epsilons() {
        let s = build(&[(Action::Fold, 0.0), (Action::Check, 0.0), (Action::Call(10), 100.0)]);
        let p = s.policy();
        assert!(close(sums(&s), 1.0));
        assert!(p[&Edge::from(Action::Call(10))] > 0.99);
    }

    #[test]
    fn clamping() {
        let s = build(&[(Action::Fold, 0.0), (Action::Check, 0.0)]);
        let p = s.policy();
        assert!(close(sums(&s), 1.0));
        assert!(close(p[&Edge::from(Action::Fold)], 0.5));
        assert!(close(p[&Edge::from(Action::Check)], 0.5));
    }

    #[test]
    fn proportionality() {
        let s = build(&[(Action::Fold, 25.0), (Action::Check, 75.0)]);
        let p = s.policy();
        assert!(close(p[&Edge::from(Action::Fold)], 0.25));
        assert!(close(p[&Edge::from(Action::Check)], 0.75));
    }

    #[test]
    fn density() {
        let s = build(&[(Action::Fold, 10.0), (Action::Check, 30.0), (Action::Call(10), 60.0)]);
        let p = s.policy();
        for (e, v) in &p {
            assert!(close(s.density(&NlheEdge::from(*e)), *v));
        }
    }

    #[test]
    fn unsupported() {
        let s = build(&[(Action::Fold, 50.0), (Action::Check, 50.0)]);
        assert_eq!(s.density(&NlheEdge::from(Action::Call(10))), 0.0);
    }

    #[test]
    fn construction() {
        let i = NlheInfo::random();
        let d: Vec<Decision<NlheEdge>> = i
            .choices()
            .into_iter()
            .map(NlheEdge::from)
            .enumerate()
            .map(|(idx, edge)| Decision {
                edge,
                mass: (idx + 1) as f32 * 10.0,
                visits: idx as u32,
                payoff: 0.0,
            })
            .collect();
        let expected = d.len();
        let s = Strategy::from((i, d));
        assert_eq!(s.info(), &i);
        assert_eq!(s.accumulated().len(), expected);
        assert_eq!(s.visits().len(), expected);
        assert!(close(sums(&s), 1.0));
    }

    #[test]
    fn singularity() {
        let s = build(&[(Action::Fold, 42.0)]);
        assert!(close(s.policy()[&Edge::from(Action::Fold)], 1.0));
    }

    #[test]
    fn support() {
        let s = build(&[(Action::Fold, 1.0), (Action::Check, 1.0), (Action::Call(10), 1.0)]);
        assert_eq!(s.support().count(), 3);
    }

    #[test]
    fn positivity() {
        let s = build(&[(Action::Fold, 5.0), (Action::Check, 15.0), (Action::Call(10), 80.0)]);
        for &p in s.policy().values() {
            assert!(p > 0.0 && p <= 1.0);
        }
    }
}
