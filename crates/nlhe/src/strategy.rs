use super::*;
use rbp_core::Probability;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_transport::*;
use std::collections::BTreeMap;

// TODO: Import from rbp-core or define locally
const POLICY_MIN: Probability = 1e-6;

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
    counts: BTreeMap<Edge, u32>,
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
    /// Encounter counts per action.
    pub fn counts(&self) -> &BTreeMap<Edge, u32> {
        &self.counts
    }
    /// Normalized action probabilities (sums to 1).
    /// Applies minimum probability floor to prevent zero weights.
    pub fn policy(&self) -> BTreeMap<Edge, Probability> {
        let denom = self
            .accumulated
            .values()
            .map(|&p| p.max(POLICY_MIN))
            .sum::<Probability>();
        self.accumulated
            .iter()
            .map(|(&edge, &policy)| (edge, policy.max(POLICY_MIN) / denom))
            .collect()
    }
}

impl Density for Strategy {
    type Support = NlheEdge;
    fn density(&self, edge: &Self::Support) -> Probability {
        let denom = self
            .accumulated
            .values()
            .map(|&p| p.max(POLICY_MIN))
            .sum::<Probability>();
        self.accumulated
            .get(edge.as_ref())
            .map(|&p| p.max(POLICY_MIN) / denom)
            .unwrap_or(0.)
    }
    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.accumulated.keys().copied().map(NlheEdge::from)
    }
}

impl From<(NlheInfo, Vec<Decision<NlheEdge>>)> for Strategy {
    fn from((info, decisions): (NlheInfo, Vec<Decision<NlheEdge>>)) -> Self {
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
        let counts = info
            .choices()
            .into_iter()
            .map(|edge| {
                (
                    edge,
                    decisions
                        .iter()
                        .find(|d| Edge::from(d.edge) == edge)
                        .map(|d| d.counts)
                        .expect("empty decision tree"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        Self { info, accumulated, counts }
    }
}

impl TryFrom<ApiStrategy> for Strategy {
    type Error = anyhow::Error;
    fn try_from(api: ApiStrategy) -> Result<Self, Self::Error> {
        let subgame = Path::from(api.history as u64);
        let present = Abstraction::from(api.present);
        let choices = Path::from(api.choices as u64);
        let info = NlheInfo::from((subgame, present, choices));
        let accumulated = api
            .accumulated
            .into_iter()
            .map(|(edge_str, policy)| Edge::try_from(edge_str.as_str()).map(|edge| (edge, policy)))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let counts = api
            .counts
            .into_iter()
            .map(|(edge_str, count)| Edge::try_from(edge_str.as_str()).map(|edge| (edge, count)))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(Self { info, accumulated, counts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::Arbitrary;

    fn build(data: &[(Action, Probability)]) -> Strategy {
        let accumulated = data.iter().map(|(a, p)| (Edge::from(*a), *p)).collect();
        let counts = data.iter().map(|(a, _)| (Edge::from(*a), 0u32)).collect();
        let info = NlheInfo::random();
        Strategy { info, accumulated, counts }
    }
    fn sums(s: &Strategy) -> Probability {
        s.policy().values().sum()
    }
    fn close(a: Probability, b: Probability) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn unitarity() {
        let s = build(&[
            (Action::Fold, 10.0),
            (Action::Check, 20.0),
            (Action::Call(10), 30.0),
        ]);
        assert!(close(sums(&s), 1.0));
    }

    #[test]
    fn epsilons() {
        let s = build(&[
            (Action::Fold, 0.0),
            (Action::Check, 0.0),
            (Action::Call(10), 100.0),
        ]);
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
        let s = build(&[
            (Action::Fold, 10.0),
            (Action::Check, 30.0),
            (Action::Call(10), 60.0),
        ]);
        let p = s.policy();
        for (e, v) in p.iter() {
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
                counts: idx as u32,
            })
            .collect();
        let expected = d.len();
        let s = Strategy::from((i.clone(), d));
        assert_eq!(s.info(), &i);
        assert_eq!(s.accumulated().len(), expected);
        assert_eq!(s.counts().len(), expected);
        assert!(close(sums(&s), 1.0));
    }

    #[test]
    fn singularity() {
        let s = build(&[(Action::Fold, 42.0)]);
        assert!(close(s.policy()[&Edge::from(Action::Fold)], 1.0));
    }

    #[test]
    fn support() {
        let s = build(&[
            (Action::Fold, 1.0),
            (Action::Check, 1.0),
            (Action::Call(10), 1.0),
        ]);
        assert_eq!(s.support().count(), 3);
    }

    #[test]
    fn positivity() {
        let s = build(&[
            (Action::Fold, 5.0),
            (Action::Check, 15.0),
            (Action::Call(10), 80.0),
        ]);
        for (_, &p) in s.policy().iter() {
            assert!(p > 0.0 && p <= 1.0);
        }
    }
}
