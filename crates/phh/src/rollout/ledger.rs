use super::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// A scored set of duels — the Tier 3 counterpart to [`crate::Agreement`].
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    duels: Vec<Duel>,
}
impl FromIterator<Duel> for Ledger {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Duel>,
    {
        Self {
            duels: iter.into_iter().collect(),
        }
    }
}
impl Ledger {
    pub fn duels(&self) -> &[Duel] {
        &self.duels
    }

    pub fn n(&self) -> usize {
        self.duels.len()
    }

    pub fn on(&self, street: Street) -> Self {
        self.duels().iter().copied().filter(|d| d.street() == street).collect()
    }

    /// Mean big blinds the logged decisions gained over ours.
    pub fn delta(&self) -> Utility {
        self.mean(Duel::delta) as Utility
    }

    /// Population standard deviation of the per-decision gain, in big blinds.
    pub fn spread(&self) -> Utility {
        let mean = self.mean(Duel::delta);
        self.mean(|d| (f64::from(d.delta()) - mean).powi(2) as Utility).sqrt() as Utility
    }

    /// [`delta`](Self::delta) per hundred decisions — the headline. Not bb/100
    /// hands: these are decisions, and a hand holds several.
    pub fn per_hundred(&self) -> Utility {
        100.0 * self.delta()
    }

    /// Standard error of [`delta`](Self::delta), in big blinds.
    pub fn stderr(&self) -> Utility {
        match self.n() {
            0 | 1 => 0.0,
            n => self.spread() / ((n - 1) as Utility).sqrt(),
        }
    }

    /// [`delta`](Self::delta) in standard errors. Inside ±2 the sign is noise.
    pub fn t(&self) -> f64 {
        match self.stderr() {
            0.0 => 0.0,
            e => f64::from(self.delta() / e),
        }
    }

    /// How many worlds every decision was averaged over, in total.
    pub fn worlds(&self) -> usize {
        self.duels()
            .iter()
            .map(|d| d.theirs().worlds() + d.ours().worlds())
            .sum()
    }

    /// Rollout decisions where the strategy had no distribution at all.
    pub fn misses(&self) -> usize {
        self.duels()
            .iter()
            .map(|d| d.theirs().misses() + d.ours().misses())
            .sum()
    }

    /// Per-edge: how often the log played it, and what playing it gained. Where
    /// the strategy is actually losing chips, rather than merely disagreeing.
    pub fn by_edge(&self) -> Vec<(Edge, usize, Utility)> {
        let mut edges = self.duels().iter().map(Duel::played).collect::<Vec<_>>();
        edges.sort_unstable();
        edges.dedup();
        let mut rows = edges
            .into_iter()
            .map(|edge| {
                let on = self
                    .duels()
                    .iter()
                    .copied()
                    .filter(|d| d.played() == edge)
                    .collect::<Self>();
                (edge, on.n(), on.delta())
            })
            .collect::<Vec<_>>();
        rows.sort_by_key(|(_, n, _)| std::cmp::Reverse(*n));
        rows
    }

    /// Population mean of a per-duel quantity, accumulated wide so a few
    /// thousand `f32` samples do not drift.
    fn mean(&self, value: impl Fn(&Duel) -> Utility) -> f64 {
        match self.duels().len() {
            0 => 0.0,
            n => self.duels().iter().map(|d| f64::from(value(d))).sum::<f64>() / n as f64,
        }
    }
}
