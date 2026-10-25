use super::*;
use deuce::*;
use kicker::*;

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
        self.verdicts()
            .iter()
            .filter(|v| v.street() == street)
            .cloned()
            .collect()
    }

    /// Fraction where our argmax matched what was played.
    pub fn rate(&self) -> f64 {
        self.share(Verdict::agrees)
    }

    /// Agreement from always predicting the most common played action — the
    /// floor any real strategy has to clear.
    pub fn naive(&self) -> f64 {
        let mut played = self.verdicts().iter().map(Verdict::played).collect::<Vec<_>>();
        played.sort_unstable();
        played.dedup();
        played
            .into_iter()
            .map(|edge| self.share(|v| v.played() == edge))
            .fold(0.0, f64::max)
    }

    /// Mean number of edges the blueprint was choosing between.
    pub fn spread(&self) -> f64 {
        self.mean(|v| v.choices() as f64)
    }

    /// `exp` of the mean negative log-likelihood: the effective number of
    /// actions our blueprint was spreading over. Lower is sharper. Compare
    /// against [`spread`](Self::spread) — matching it means we predicted no
    /// better than a coin over the legal actions.
    pub fn perplexity(&self) -> f64 {
        self.mean(|v| -f64::from(v.likelihood().max(FLOOR)).ln()).exp()
    }

    /// Spots where we put no mass at all on what was played.
    pub fn impossible(&self) -> usize {
        self.verdicts().iter().filter(|v| v.likelihood() <= 0.0).count()
    }

    /// Played-versus-chosen counts, heaviest first. Where the disagreements
    /// actually live.
    pub fn confusion(&self) -> Vec<(Edge, Edge, usize)> {
        let mut counts = std::collections::BTreeMap::<(Edge, Edge), usize>::new();
        self.verdicts()
            .iter()
            .for_each(|v| *counts.entry((v.played(), v.chosen())).or_default() += 1);
        let mut rows = counts.into_iter().map(|((p, c), n)| (p, c, n)).collect::<Vec<_>>();
        rows.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));
        rows
    }

    /// How often each edge was played, and how often we chose it. A blueprint
    /// that systematically under- or over-uses a size shows up here even when
    /// per-spot agreement looks reasonable.
    pub fn bias(&self) -> Vec<(Edge, f64, f64)> {
        let mut edges = self
            .verdicts()
            .iter()
            .flat_map(|v| [v.played(), v.chosen()])
            .collect::<Vec<_>>();
        edges.sort_unstable();
        edges.dedup();
        edges
            .into_iter()
            .map(|edge| (edge, self.share(|v| v.played() == edge), self.share(|v| v.chosen() == edge)))
            .collect()
    }

    fn share(&self, test: impl Fn(&Verdict) -> bool) -> f64 {
        self.mean(|v| f64::from(u8::from(test(v))))
    }

    fn mean(&self, value: impl Fn(&Verdict) -> f64) -> f64 {
        match self.verdicts().len() {
            0 => 0.0,
            n => self.verdicts().iter().map(value).sum::<f64>() / n as f64,
        }
    }
}
