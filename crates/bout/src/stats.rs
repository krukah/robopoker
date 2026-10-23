//! Sample statistics for a stream of per-hand results.

/// Mean, spread, and interval for one measured quantity over a run.
///
/// Everything a benchmark reports is this shape applied to a different projection of the same
/// hands — all of them, hero's hands on the button, the opponent's luck-adjusted estimate — so
/// the arithmetic lives here once and the benchmark just chooses what to feed it.
///
/// Rates are per 100 hands, the unit every poker result is quoted in.
pub struct Stats {
    hands: usize,
    total: f64,
    stddev: f64,
}

impl Stats {
    /// Summarize a sample. Fewer than two observations have no spread, so
    /// their interval is zero rather than undefined.
    pub fn of(sample: impl Iterator<Item = f64>) -> Self {
        let sample = sample.collect::<Vec<f64>>();
        let hands = sample.len();
        let total = sample.iter().sum::<f64>();
        let stddev = match hands {
            0 | 1 => 0.0,
            n => (sample.iter().map(|x| (x - total / n as f64).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt(),
        };
        Self { hands, total, stddev }
    }

    /// Observations in the sample.
    pub fn hands(&self) -> usize {
        self.hands
    }

    /// Sum over the sample, in bb.
    pub fn total(&self) -> f64 {
        self.total
    }

    /// Mean rate in bb per 100 hands.
    pub fn rate(&self) -> f64 {
        match self.hands() {
            0 => 0.0,
            n => self.total() / n as f64 * 100.0,
        }
    }

    /// Sample standard deviation, in bb per hand.
    pub fn stddev(&self) -> f64 {
        self.stddev
    }

    /// Half-width of the 95% confidence interval on [`rate`](Stats::rate).
    pub fn confidence(&self) -> f64 {
        match self.hands() {
            0 => 0.0,
            n => 1.96 * self.stddev() / (n as f64).sqrt() * 100.0,
        }
    }

    /// Lower bound of that interval — the pessimistic read, and the figure
    /// a leaderboard should rank on.
    pub fn lower(&self) -> f64 {
        self.rate() - self.confidence()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty sample reports zeros, never NaN — a benchmark that errored on
    /// every hand still has to print a report.
    #[test]
    fn empty_sample_is_zero() {
        let stats = Stats::of(std::iter::empty());
        assert_eq!(stats.hands(), 0);
        assert_eq!(stats.rate(), 0.0);
        assert_eq!(stats.confidence(), 0.0);
    }

    /// One observation has a rate but no spread.
    #[test]
    fn single_sample_has_no_spread() {
        let stats = Stats::of(std::iter::once(-3.0));
        assert!((stats.rate() - -300.0).abs() < f64::EPSILON);
        assert_eq!(stats.stddev(), 0.0);
    }

    /// Rate is the per-hand mean scaled to 100 hands, and the lower bound
    /// sits a confidence half-width below it.
    #[test]
    fn rate_scales_to_per_hundred() {
        let stats = Stats::of([2.0, -1.0, 2.0, -1.0].into_iter());
        assert!((stats.rate() - 50.0).abs() < 1e-9);
        assert!((stats.lower() - (stats.rate() - stats.confidence())).abs() < f64::EPSILON);
    }
}
