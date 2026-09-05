//! Tier 2: does our blueprint choose what Pluribus chose?
//!
//! Scoring is deliberately modest about what it can prove. Agreement with
//! Pluribus is not correctness — Pluribus is not a Nash equilibrium and was
//! playing a different game. A *low* agreement rate on a spot type is a lead
//! worth pulling; a high one is weak evidence of nothing being badly wrong.
//!
//! Two numbers do the work:
//!
//! - **Agreement** — how often our argmax matches. Cheap to read, but it throws
//!   away the distribution and rewards being confidently wrong less than it
//!   should.
//! - **Perplexity** — `exp` of the mean negative log-likelihood of Pluribus's
//!   actual action under our distribution. Reads as *"our blueprint was
//!   effectively choosing uniformly among this many actions"*. Unlike agreement
//!   it is sensitive to how much mass we put on the played action, so a policy
//!   that hedges correctly beats one that guesses right half the time and
//!   assigns zero the rest.
//!
//! Neither means anything without a floor to compare against, so [`Agreement`]
//! carries two: [`Agreement::naive`], the score from always predicting the
//! single most common action, and [`Agreement::spread`], the mean size of the
//! choice set. A blueprint that cannot beat `naive` has told us nothing.

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Likelihood floor, so one impossible action cannot swallow the mean.
///
/// A played action our blueprint assigns zero mass scores as if it had this
/// much — about a 1-in-1000 call. [`Agreement::impossible`] counts them so the
/// floor never hides how often it fired.
const FLOOR: Probability = 0.001;

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
        self.played == self.chosen
    }
}

/// A scored set of spots.
#[derive(Debug, Clone, Default)]
pub struct Agreement {
    verdicts: Vec<Verdict>,
}

impl FromIterator<Verdict> for Agreement {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Verdict>,
    {
        Self {
            verdicts: iter.into_iter().collect(),
        }
    }
}

impl Agreement {
    pub fn verdicts(&self) -> &[Verdict] {
        &self.verdicts
    }

    pub fn n(&self) -> usize {
        self.verdicts.len()
    }

    pub fn on(&self, street: Street) -> Self {
        self.verdicts.iter().filter(|v| v.street == street).cloned().collect()
    }

    /// Fraction where our argmax matched what was played.
    pub fn rate(&self) -> f64 {
        self.share(Verdict::agrees)
    }

    /// Agreement from always predicting the most common played action — the
    /// floor any real strategy has to clear.
    pub fn naive(&self) -> f64 {
        let mut played = self.verdicts.iter().map(|v| v.played).collect::<Vec<_>>();
        played.sort_unstable();
        played.dedup();
        played
            .into_iter()
            .map(|edge| self.share(|v| v.played == edge))
            .fold(0.0, f64::max)
    }

    /// Mean number of edges the blueprint was choosing between.
    pub fn spread(&self) -> f64 {
        self.mean(|v| v.choices as f64)
    }

    /// `exp` of the mean negative log-likelihood: the effective number of
    /// actions our blueprint was spreading over. Lower is sharper. Compare
    /// against [`spread`](Self::spread) — matching it means we predicted no
    /// better than a coin over the legal actions.
    pub fn perplexity(&self) -> f64 {
        self.mean(|v| -f64::from(v.likelihood.max(FLOOR)).ln()).exp()
    }

    /// Spots where we put no mass at all on what was played.
    pub fn impossible(&self) -> usize {
        self.verdicts.iter().filter(|v| v.likelihood <= 0.0).count()
    }

    /// Played-versus-chosen counts, heaviest first. Where the disagreements
    /// actually live.
    pub fn confusion(&self) -> Vec<(Edge, Edge, usize)> {
        let mut counts = std::collections::BTreeMap::<(Edge, Edge), usize>::new();
        self.verdicts
            .iter()
            .for_each(|v| *counts.entry((v.played, v.chosen)).or_default() += 1);
        let mut rows = counts.into_iter().map(|((p, c), n)| (p, c, n)).collect::<Vec<_>>();
        rows.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));
        rows
    }

    /// How often each edge was played, and how often we chose it. A blueprint
    /// that systematically under- or over-uses a size shows up here even when
    /// per-spot agreement looks reasonable.
    pub fn bias(&self) -> Vec<(Edge, f64, f64)> {
        let mut edges = self
            .verdicts
            .iter()
            .flat_map(|v| [v.played, v.chosen])
            .collect::<Vec<_>>();
        edges.sort_unstable();
        edges.dedup();
        edges
            .into_iter()
            .map(|edge| (edge, self.share(|v| v.played == edge), self.share(|v| v.chosen == edge)))
            .collect()
    }

    fn share(&self, test: impl Fn(&Verdict) -> bool) -> f64 {
        self.mean(|v| f64::from(u8::from(test(v))))
    }

    fn mean(&self, value: impl Fn(&Verdict) -> f64) -> f64 {
        match self.verdicts.len() {
            0 => 0.0,
            n => self.verdicts.iter().map(value).sum::<f64>() / n as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verdict(played: Edge, chosen: Edge, likelihood: Probability) -> Verdict {
        Verdict {
            street: Street::Flop,
            played,
            chosen,
            likelihood,
            choices: 4,
        }
    }

    #[test]
    fn agreement_counts_argmax_matches() {
        let a = [
            verdict(Edge::Check, Edge::Check, 0.8),
            verdict(Edge::Check, Edge::Fold, 0.2),
            verdict(Edge::Fold, Edge::Fold, 0.9),
            verdict(Edge::Fold, Edge::Fold, 0.7),
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
            .map(|_| verdict(Edge::Check, Edge::Check, 0.9))
            .collect::<Agreement>();
        let vague = (0..4)
            .map(|_| verdict(Edge::Check, Edge::Check, 0.3))
            .collect::<Agreement>();
        assert_eq!(sharp.rate(), vague.rate());
        assert!(sharp.perplexity() < vague.perplexity());
        assert!((sharp.perplexity() - 1.0 / 0.9).abs() < 1e-6);
    }

    #[test]
    fn zero_mass_is_floored_and_counted() {
        let a = [verdict(Edge::Check, Edge::Fold, 0.0)]
            .into_iter()
            .collect::<Agreement>();
        assert_eq!(a.impossible(), 1);
        assert!(a.perplexity().is_finite());
        assert!((a.perplexity() - 1.0 / f64::from(FLOOR)).abs() < 1.0);
    }

    #[test]
    fn confusion_is_ordered_by_weight() {
        let a = [
            verdict(Edge::Check, Edge::Fold, 0.1),
            verdict(Edge::Check, Edge::Fold, 0.1),
            verdict(Edge::Fold, Edge::Fold, 0.9),
        ]
        .into_iter()
        .collect::<Agreement>();
        assert_eq!(a.confusion()[0], (Edge::Check, Edge::Fold, 2));
    }

    #[test]
    fn bias_shows_over_and_under_use() {
        let a = [
            verdict(Edge::Check, Edge::Fold, 0.1),
            verdict(Edge::Check, Edge::Fold, 0.1),
            verdict(Edge::Check, Edge::Check, 0.8),
        ]
        .into_iter()
        .collect::<Agreement>();
        let rows = a.bias();
        let check = rows.iter().find(|(e, _, _)| *e == Edge::Check).unwrap();
        assert!((check.1 - 1.0).abs() < 1e-9, "played every time");
        assert!((check.2 - 1.0 / 3.0).abs() < 1e-9, "chosen a third of the time");
    }
}
