#![doc = include_str!("../doc/agree.md")]

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Likelihood floor, so one impossible action cannot swallow the mean.
///
/// A played action our blueprint assigns zero mass scores as if it had this
/// much — about a 1-in-1000 call. [`Agreement::impossible`] counts them so the
/// floor never hides how often it fired.
pub(crate) const FLOOR: Probability = 0.001;

/// What the blueprint said, against what was played, at one spot.
#[derive(Debug, Clone)]
pub struct Verdict {
    street: Street,
    played: Edge,
    chosen: Edge,
    likelihood: Probability,
    choices: usize,
}

impl Verdict {
    /// Scores one spot against a strategy's distribution over edges.
    pub fn new(spot: &Spot, policy: &Policy) -> Option<Self> {
        Some(Self {
            street: spot.street(),
            played: spot.edge(),
            chosen: policy.favorite()?,
            likelihood: policy.mass(spot.edge()),
            choices: policy.len(),
        })
    }

    pub fn street(&self) -> Street {
        self.street
    }

    /// The edge Pluribus's action snapped to.
    pub fn played(&self) -> Edge {
        self.played
    }

    /// The edge our blueprint would have taken.
    pub fn chosen(&self) -> Edge {
        self.chosen
    }

    /// Our probability on the action actually played.
    pub fn likelihood(&self) -> Probability {
        self.likelihood
    }

    pub fn agrees(&self) -> bool {
        self.played() == self.chosen()
    }
}

impl Verdict {
    pub fn choices(&self) -> usize {
        self.choices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Verdict {
        fn flop(played: Edge, chosen: Edge, likelihood: Probability) -> Self {
            Self {
                street: Street::Flop,
                played,
                chosen,
                likelihood,
                choices: 4,
            }
        }
    }

    #[test]
    fn agreement_counts_argmax_matches() {
        let a = [
            Verdict::flop(Edge::Check, Edge::Check, 0.8),
            Verdict::flop(Edge::Check, Edge::Fold, 0.2),
            Verdict::flop(Edge::Fold, Edge::Fold, 0.9),
            Verdict::flop(Edge::Fold, Edge::Fold, 0.7),
        ]
        .into_iter()
        .collect::<Agreement>();
        assert_eq!(a.rate(), 0.75);
        // Fold and Check were played twice each, so guessing either scores 50%.
        assert_eq!(a.naive(), 0.5);
    }

    /// Perplexity has to reward putting mass on what happened, not just being
    /// right on the argmax.
    #[test]
    fn perplexity_separates_confident_from_lucky() {
        let sharp = (0..4)
            .map(|_| Verdict::flop(Edge::Check, Edge::Check, 0.9))
            .collect::<Agreement>();
        let vague = (0..4)
            .map(|_| Verdict::flop(Edge::Check, Edge::Check, 0.3))
            .collect::<Agreement>();
        assert_eq!(sharp.rate(), vague.rate());
        assert!(sharp.perplexity() < vague.perplexity());
        assert!((sharp.perplexity() - 1.0 / 0.9).abs() < 1e-6);
    }

    #[test]
    fn zero_mass_is_floored_and_counted() {
        let a = [Verdict::flop(Edge::Check, Edge::Fold, 0.0)]
            .into_iter()
            .collect::<Agreement>();
        assert_eq!(a.impossible(), 1);
        assert!(a.perplexity().is_finite());
        assert!((a.perplexity() - 1.0 / f64::from(FLOOR)).abs() < 1.0);
    }

    #[test]
    fn confusion_is_ordered_by_weight() {
        let a = [
            Verdict::flop(Edge::Check, Edge::Fold, 0.1),
            Verdict::flop(Edge::Check, Edge::Fold, 0.1),
            Verdict::flop(Edge::Fold, Edge::Fold, 0.9),
        ]
        .into_iter()
        .collect::<Agreement>();
        assert_eq!(a.confusion()[0], (Edge::Check, Edge::Fold, 2));
    }

    #[test]
    fn bias_shows_over_and_under_use() {
        let a = [
            Verdict::flop(Edge::Check, Edge::Fold, 0.1),
            Verdict::flop(Edge::Check, Edge::Fold, 0.1),
            Verdict::flop(Edge::Check, Edge::Check, 0.8),
        ]
        .into_iter()
        .collect::<Agreement>();
        let rows = a.bias();
        let check = rows.iter().find(|(e, _, _)| *e == Edge::Check).unwrap();
        assert!((check.1 - 1.0).abs() < 1e-9, "played every time");
        assert!((check.2 - 1.0 / 3.0).abs() < 1e-9, "chosen a third of the time");
    }
}
