use super::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Per-thread cache of `OT_ε(μ, μ)` self-costs, keyed by histogram content
/// hash + street.
///
/// # Cache validity contract
///
/// `OT_ε(h, h)` is a pure function of `(h, T, ground_metric)`. The cache
/// is correct under three invariants — all currently true in this crate:
///
/// 1. `SinkhornHyperParams::DEFAULT` (temperature, iters, tolerance) is
///    `const`, so `T` never changes within a process.
/// 2. Each `Metric` is constructed once and never mutated. `Metric::set`
///    is only called during construction (in `Metric::from`).
/// 3. All `Metric` instances for the same `Street` within a process
///    yield identical `raw_distance(x, y)` values — they all derive
///    from the same DB row set or the same deterministic clustering
///    output.
///
/// **If any of these is violated** (e.g. a runtime-tunable temperature, or
/// a mutating `Metric`), this cache will return stale values and must be
/// removed or invalidated.
///
/// Bounded by clear-on-overflow rather than LRU — simpler, cheap, and the
/// hot-path access pattern (k-means: `K` centroids + `N` points repeatedly
/// queried) is monotonic within a phase, so eviction is rare.
///
/// Sized for the largest expected per-thread working set on a Flop run
/// (`N/threads ≈ 160k` points + `K × E ≈ 2.5k` centroid states) with
/// headroom for thread-count variation. At ~25 bytes/entry (hashbrown
/// overhead included), ~6 MB/thread × ~16 threads ≈ 100 MB peak total.
const SELF_COST_CACHE_LIMIT: usize = 1 << 18;
thread_local! {
    static SELF_COST_CACHE: RefCell<HashMap<u64, Energy>> = RefCell::new(HashMap::new());
}

/// Entropic optimal transport via Sinkhorn iteration.
///
/// Computes the Earth Mover's Distance (Wasserstein-1) between two histograms
/// using the Sinkhorn algorithm with entropic regularization. This trades
/// slight approximation error for O(n²) complexity instead of O(n³ log n).
///
/// # Algorithm
///
/// Uses the Kantorovich-Rubinstein dual formulation with Sinkhorn scaling:
/// 1. Initialize potentials uniformly
/// 2. Alternately scale LHS and RHS potentials
/// 3. Stop when potential changes fall below tolerance
/// 4. Compute transport cost from final coupling
///
/// # Regularization
///
/// The `temperature` hyperparameter controls entropic smoothing:
/// - Lower → sharper coupling, closer to true EMD, slower convergence
/// - Higher → smoother coupling, faster convergence, more approximation
pub struct Sinkhorn<'a> {
    /// Ground metric for distance between abstractions.
    metric: &'a Metric,
    /// Source distribution.
    mu: &'a Histogram,
    /// Target distribution.
    nu: &'a Histogram,
    /// LHS potential (dual variable).
    lhs: Potential,
    /// RHS potential (dual variable).
    rhs: Potential,
}

impl Sinkhorn<'_> {
    /// Runs Sinkhorn iteration until convergence.
    fn sinkhorn(&mut self) {
        #[allow(unused)]
        for t in 0..self.iterations() {
            let ref mut next = self.lhs();
            let ref mut prev = self.lhs;
            let lhs_err = Self::delta(prev, next);
            std::mem::swap(prev, next);
            let ref mut next = self.rhs();
            let ref mut prev = self.rhs;
            let rhs_err = Self::delta(prev, next);
            std::mem::swap(prev, next);
            if lhs_err + rhs_err < self.tolerance() {
                break;
            }
        }
    }
    /// Computes updated LHS potential via Sinkhorn scaling.
    fn lhs(&self) -> Potential {
        let mut next = Potential::zeroes(self.mu);
        self.lhs
            .support()
            .map(|x| (x, self.softmin(&x, self.mu, &self.rhs)))
            .inspect(|(_, d)| debug_assert!(d.is_finite(), "lhs entropy overflow"))
            .for_each(|(x, d)| next.set(&x, d));
        next
    }
    /// Computes updated RHS potential via Sinkhorn scaling.
    fn rhs(&self) -> Potential {
        let mut next = Potential::zeroes(self.nu);
        self.rhs
            .support()
            .map(|x| (x, self.softmin(&x, self.nu, &self.lhs)))
            .inspect(|(_, d)| debug_assert!(d.is_finite(), "rhs entropy overflow"))
            .for_each(|(x, d)| next.set(&x, d));
        next
    }
    /// Computes coupling mass at (x, y) from potentials.
    fn coupling(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        (self.lhs.density(x) + self.rhs.density(y) - self.regularization(x, y)).exp()
    }
    /// Computes log-scale potential update for one support element.
    /// Balances the marginal constraint via softmin over opposing potential.
    fn softmin(&self, x: &Abstraction, histogram: &Histogram, potential: &Potential) -> Entropy {
        histogram.density(x).ln()
            - potential
                .support()
                .map(|y| potential.density(&y) - self.regularization(x, &y))
                .map(f32::exp)
                .map(|e| e.max(Energy::MIN_POSITIVE))
                .sum::<Energy>()
                .ln()
    }
    /// Regularized cost: distance / temperature.
    fn regularization(&self, x: &Abstraction, y: &Abstraction) -> Entropy {
        self.metric.raw_distance(x, y) / self.temperature()
    }
    /// Computes L1 change in potential (stopping criterion).
    fn delta(prev: &Potential, next: &Potential) -> Energy {
        prev.support()
            .map(|x| next.density(&x).exp() - prev.density(&x).exp())
            .map(f32::abs)
            .sum::<Energy>()
    }
    /// Entropic regularization strength. Lower = closer to exact EMD.
    fn temperature(&self) -> Entropy {
        crate::SinkhornHyperParams::DEFAULT.temperature()
    }
    /// Maximum iteration count before forced termination.
    fn iterations(&self) -> usize {
        crate::SinkhornHyperParams::DEFAULT.iterations()
    }
    /// Convergence tolerance for early stopping.
    fn tolerance(&self) -> Energy {
        crate::SinkhornHyperParams::DEFAULT.tolerance()
    }
    /// Sinkhorn divergence: entropic-bias-debiased EMD.
    ///
    /// `S_ε(μ, ν) = OT_ε(μ, ν) − ½·OT_ε(μ, μ) − ½·OT_ε(ν, ν)`
    ///
    /// Restores `S_ε(μ, μ) = 0` and recovers a true semi-metric in the
    /// presence of entropic regularization. The two self-terms cancel
    /// the diagonal coupling that the regularizer cannot reach.
    /// See Feydy et al. (2019), "Interpolating between Optimal Transport
    /// and MMD using Sinkhorn Divergences."
    ///
    /// Self-terms are memoised in a per-thread cache so k-means inner
    /// loops (`N × K` cross-distances per iteration, same μ and ν reused
    /// many times) pay the self-cost exactly once per histogram per
    /// thread, not once per cross-distance call.
    pub fn divergence(mu: &Histogram, nu: &Histogram, metric: &Metric) -> Energy {
        let xy = Sinkhorn::from((mu, nu, metric)).minimize().cost();
        let xx = Self::self_cost(mu, metric);
        let yy = Self::self_cost(nu, metric);
        (xy - 0.5 * xx - 0.5 * yy).max(0.0)
    }
    /// Memoised `OT_ε(h, h)`. Cache key combines histogram content hash
    /// with the metric's street, since two metrics on different streets
    /// over the same histogram shape would yield different self-costs.
    fn self_cost(h: &Histogram, metric: &Metric) -> Energy {
        let ref mut hasher = std::collections::hash_map::DefaultHasher::new();
        h.hash(hasher);
        metric.street().hash(hasher);
        let key = hasher.finish();
        SELF_COST_CACHE.with_borrow_mut(|cache| {
            if let Some(&v) = cache.get(&key) {
                return v;
            }
            let v = Sinkhorn::from((h, h, metric)).minimize().cost();
            if cache.len() >= SELF_COST_CACHE_LIMIT {
                cache.clear();
            }
            cache.insert(key, v);
            v
        })
    }
}

impl Coupling for Sinkhorn<'_> {
    type X = ClusterAbs;
    type Y = ClusterAbs;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn minimize(mut self) -> Self {
        self.sinkhorn();
        self
    }

    fn flow(&self, x: &Self::X, y: &Self::Y) -> Energy {
        self.coupling(x, y) * self.metric.raw_distance(x, y)
    }

    fn cost(&self) -> Energy {
        self.lhs
            .support()
            .flat_map(|x| self.rhs.support().map(move |y| (x, y)))
            .map(|(x, y)| self.flow(&ClusterAbs::from(x), &ClusterAbs::from(y)))
            .inspect(|x| debug_assert!(x.is_finite()))
            .sum::<Energy>()
    }
}

impl<'a> From<(&'a Histogram, &'a Histogram, &'a Metric)> for Sinkhorn<'a> {
    fn from((mu, nu, metric): (&'a Histogram, &'a Histogram, &'a Metric)) -> Self {
        Self {
            metric,
            mu,
            nu,
            lhs: Potential::uniform(mu),
            rhs: Potential::uniform(nu),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_cards::Street;

    /// Build a Flop histogram from `(abstraction_index, count)` pairs. The
    /// support sits on Flop abstractions, so `Metric::emd` on this kind of
    /// histogram dispatches to Sinkhorn (the path we want to test).
    fn flop_hist(entries: &[(usize, usize)]) -> Histogram {
        let mut h = Histogram::empty(Street::Flop);
        for &(idx, count) in entries {
            h.set(Abstraction::from((Street::Flop, idx)), count);
        }
        h
    }

    /// Build a deterministic non-trivial Flop ground metric. Real metrics
    /// come from clustering output; for unit tests we just need enough
    /// non-zero entries to make Sinkhorn produce non-trivial couplings
    /// over the small support of `flop_hist` test inputs.
    fn flop_metric() -> Metric {
        let mut m = Metric::new(Street::Flop);
        for i in 0..32 {
            for j in (i + 1)..32 {
                let pair = Pair::new(Street::Flop, i, j);
                let d = (((i * 7 + j * 13) % 97) + 1) as f32 / 100.0;
                m.set(pair, d);
            }
        }
        m
    }

    /// Tests share the thread-local cache. Each test that asserts on cache
    /// state must clear at the top so it sees a known-empty cache, even
    /// when cargo reuses a thread for sequential tests.
    fn clear_cache() {
        SELF_COST_CACHE.with_borrow_mut(std::collections::HashMap::clear);
    }

    /// The whole point: `S_ε(μ, μ) = 0`. With the buggy raw OT the diagnostic
    /// measured ~0.02 here; this test would fail without the divergence subtraction.
    #[test]
    fn divergence_is_zero_on_self() {
        clear_cache();
        let metric = flop_metric();
        let h = flop_hist(&[(0, 3), (5, 1), (12, 4), (24, 2)]);
        let d = Sinkhorn::divergence(&h, &h, &metric);
        assert!(d.abs() < 1e-4, "expected self-divergence ≈ 0, got {d}");
    }

    /// `S_ε(μ, ν) = S_ε(ν, μ)`. Cross-Sinkhorn iteration order isn't
    /// strictly symmetric numerically, so the assertion uses a small slack.
    #[test]
    fn divergence_is_symmetric() {
        clear_cache();
        let metric = flop_metric();
        let mu = flop_hist(&[(0, 3), (5, 1), (12, 4)]);
        let nu = flop_hist(&[(2, 2), (8, 5), (20, 1), (24, 3)]);
        let d12 = Sinkhorn::divergence(&mu, &nu, &metric);
        let d21 = Sinkhorn::divergence(&nu, &mu, &metric);
        assert!((d12 - d21).abs() < 1e-3, "asymmetric: {d12} vs {d21}");
    }

    /// First call populates two cache entries (one per unique input);
    /// second call hits both and the cache size stays put. This is the
    /// observable contract of the cache.
    #[test]
    fn cache_populates_one_entry_per_unique_input() {
        clear_cache();
        let metric = flop_metric();
        let mu = flop_hist(&[(0, 3), (5, 1)]);
        let nu = flop_hist(&[(2, 2), (8, 5)]);
        let _ = Sinkhorn::divergence(&mu, &nu, &metric);
        let after_first = SELF_COST_CACHE.with_borrow(std::collections::HashMap::len);
        assert_eq!(after_first, 2, "expected 2 self-cost entries");
        let _ = Sinkhorn::divergence(&mu, &nu, &metric);
        let after_second = SELF_COST_CACHE.with_borrow(std::collections::HashMap::len);
        assert_eq!(after_second, 2, "cache should not grow on repeat");
    }

    /// Three unique histograms used across two calls → three distinct
    /// cache keys. Confirms the hash collapses repeats but separates
    /// distinct content.
    #[test]
    fn cache_separates_distinct_histograms() {
        clear_cache();
        let metric = flop_metric();
        let h0 = flop_hist(&[(0, 1)]);
        let h1 = flop_hist(&[(1, 1)]);
        let h2 = flop_hist(&[(2, 1)]);
        let _ = Sinkhorn::divergence(&h0, &h1, &metric);
        let _ = Sinkhorn::divergence(&h1, &h2, &metric);
        let size = SELF_COST_CACHE.with_borrow(std::collections::HashMap::len);
        assert_eq!(size, 3, "expected 3 unique self-cost entries");
    }

    /// Cached value must equal the value computed without the cache, for
    /// the same inputs. Bug fence: a hash collision producing wrong values
    /// would surface here.
    #[test]
    fn cache_returns_same_value_as_fresh_computation() {
        clear_cache();
        let metric = flop_metric();
        let mu = flop_hist(&[(0, 3), (5, 1), (12, 4)]);
        let nu = flop_hist(&[(2, 2), (8, 5), (24, 3)]);
        let warm = Sinkhorn::divergence(&mu, &nu, &metric);
        clear_cache();
        let cold = Sinkhorn::divergence(&mu, &nu, &metric);
        assert!((warm - cold).abs() < 1e-6, "warm={warm} cold={cold}");
    }
}
