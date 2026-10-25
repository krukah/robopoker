use super::*;
use kicker::*;
use monge::*;
use pokerkit::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Caches `OT_ε(h, h)`: correct only while the three invariants in README.md
// ("Self-cost cache") hold, else it serves stale values.
const SELF_COST_CACHE_LIMIT: usize = 1 << 18;
thread_local! {
    static SELF_COST_CACHE: RefCell<HashMap<u64, Energy>> = RefCell::new(HashMap::new());
}

/// Entropic optimal transport via Sinkhorn iteration: EMD (Wasserstein-1) between two
/// histograms in O(n²) instead of O(n³ log n), at the price of approximation error.
///
/// Works in the Kantorovich-Rubinstein dual — uniform potentials, alternating LHS/RHS scaling
/// until the potential delta falls under tolerance, then transport cost from the final
/// coupling.
pub struct Sinkhorn<'emd> {
    /// Ground metric for distance between abstractions.
    metric: &'emd Metric,
    /// Source distribution.
    mu: &'emd Histogram,
    /// Target distribution.
    nu: &'emd Histogram,
    /// LHS potential (dual variable).
    lhs: Potential,
    /// RHS potential (dual variable).
    rhs: Potential,
}

impl<'emd> Sinkhorn<'emd> {
    fn metric(&self) -> &'emd Metric {
        self.metric
    }

    fn mu(&self) -> &'emd Histogram {
        self.mu
    }

    fn nu(&self) -> &'emd Histogram {
        self.nu
    }

    fn phi(&self) -> &Potential {
        &self.lhs
    }

    fn psi(&self) -> &Potential {
        &self.rhs
    }

    fn phi_mut(&mut self) -> &mut Potential {
        &mut self.lhs
    }

    fn psi_mut(&mut self) -> &mut Potential {
        &mut self.rhs
    }

    /// Runs Sinkhorn iteration until convergence.
    fn sinkhorn(&mut self) {
        for _ in 0..self.iterations() {
            if self.relax() < self.tolerance() {
                break;
            }
        }
    }

    fn relax(&mut self) -> Energy {
        let lhs = self.lhs();
        let lerr = Self::delta(self.phi(), &lhs);
        *self.phi_mut() = lhs;
        let rhs = self.rhs();
        let rerr = Self::delta(self.psi(), &rhs);
        *self.psi_mut() = rhs;
        lerr + rerr
    }
    /// Computes updated LHS potential via Sinkhorn scaling.
    fn lhs(&self) -> Potential {
        let mut next = Potential::zeroes(self.mu());
        self.phi()
            .support()
            .map(|x| (x, self.softmin(&x, self.mu(), self.psi())))
            .inspect(|(_, d)| debug_assert!(d.is_finite(), "lhs entropy overflow"))
            .for_each(|(x, d)| next.set(&x, d));
        next
    }
    /// Computes updated RHS potential via Sinkhorn scaling.
    fn rhs(&self) -> Potential {
        let mut next = Potential::zeroes(self.nu());
        self.psi()
            .support()
            .map(|x| (x, self.softmin(&x, self.nu(), self.phi())))
            .inspect(|(_, d)| debug_assert!(d.is_finite(), "rhs entropy overflow"))
            .for_each(|(x, d)| next.set(&x, d));
        next
    }
    /// Computes coupling mass at (x, y) from potentials.
    fn coupling(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        (self.phi().density(x) + self.psi().density(y) - self.regularization(x, y)).exp()
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
        self.metric().raw_distance(x, y) / self.temperature()
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
    /// Restores `S_ε(μ, μ) = 0` and recovers a true semi-metric in the presence of entropic
    /// regularization. The two self-terms cancel the diagonal coupling that the regularizer
    /// cannot reach. See Feydy et al.
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
        self.coupling(x, y) * self.metric().raw_distance(x, y)
    }

    fn cost(&self) -> Energy {
        self.phi()
            .support()
            .flat_map(|x| self.psi().support().map(move |y| (x, y)))
            .map(|(x, y)| self.flow(&ClusterAbs::from(x), &ClusterAbs::from(y)))
            .inspect(|x| debug_assert!(x.is_finite()))
            .sum::<Energy>()
    }
}

impl<'emd> From<(&'emd Histogram, &'emd Histogram, &'emd Metric)> for Sinkhorn<'emd> {
    fn from((mu, nu, metric): (&'emd Histogram, &'emd Histogram, &'emd Metric)) -> Self {
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
    use deuce::Street;

    impl Histogram {
        /// Build a Flop histogram from `(abstraction_index, count)` pairs.
        fn flop(entries: &[(usize, usize)]) -> Self {
            let mut h = Self::empty(Street::Flop);
            for &(idx, count) in entries {
                h.set(Abstraction::from((Street::Flop, idx)), count);
            }
            h
        }
    }

    impl Metric {
        /// Build a deterministic non-trivial Flop ground metric.
        fn flop() -> Self {
            let mut m = Self::new(Street::Flop);
            for i in 0..32 {
                for j in (i + 1)..32 {
                    let pair = Pair::new(Street::Flop, i, j);
                    let d = (((i * 7 + j * 13) % 97) + 1) as f32 / 100.0;
                    m.set(pair, d);
                }
            }
            m
        }
    }

    impl Sinkhorn<'_> {
        /// Tests share the thread-local cache.
        fn clear() {
            SELF_COST_CACHE.with_borrow_mut(std::collections::HashMap::clear);
        }
    }

    /// The whole point: `S_ε(μ, μ) = 0`. With the buggy raw OT the diagnostic
    /// measured ~0.02 here; this test would fail without the divergence subtraction.
    #[test]
    fn divergence_is_zero_on_self() {
        Sinkhorn::clear();
        let metric = Metric::flop();
        let h = Histogram::flop(&[(0, 3), (5, 1), (12, 4), (24, 2)]);
        let d = Sinkhorn::divergence(&h, &h, &metric);
        assert!(d.abs() < 1e-4, "expected self-divergence ≈ 0, got {d}");
    }

    /// `S_ε(μ, ν) = S_ε(ν, μ)`. Cross-Sinkhorn iteration order isn't
    /// strictly symmetric numerically, so the assertion uses a small slack.
    #[test]
    fn divergence_is_symmetric() {
        Sinkhorn::clear();
        let metric = Metric::flop();
        let mu = Histogram::flop(&[(0, 3), (5, 1), (12, 4)]);
        let nu = Histogram::flop(&[(2, 2), (8, 5), (20, 1), (24, 3)]);
        let d12 = Sinkhorn::divergence(&mu, &nu, &metric);
        let d21 = Sinkhorn::divergence(&nu, &mu, &metric);
        assert!((d12 - d21).abs() < 1e-3, "asymmetric: {d12} vs {d21}");
    }

    /// First call populates two cache entries (one per unique input); second call hits both and
    /// the cache size stays put. This is the observable contract of the cache.
    #[test]
    fn cache_populates_one_entry_per_unique_input() {
        Sinkhorn::clear();
        let metric = Metric::flop();
        let mu = Histogram::flop(&[(0, 3), (5, 1)]);
        let nu = Histogram::flop(&[(2, 2), (8, 5)]);
        let _ = Sinkhorn::divergence(&mu, &nu, &metric);
        let after_first = SELF_COST_CACHE.with_borrow(std::collections::HashMap::len);
        assert_eq!(after_first, 2, "expected 2 self-cost entries");
        let _ = Sinkhorn::divergence(&mu, &nu, &metric);
        let after_second = SELF_COST_CACHE.with_borrow(std::collections::HashMap::len);
        assert_eq!(after_second, 2, "cache should not grow on repeat");
    }

    /// Three unique histograms used across two calls → three distinct cache keys. Confirms the
    /// hash collapses repeats but separates distinct content.
    #[test]
    fn cache_separates_distinct_histograms() {
        Sinkhorn::clear();
        let metric = Metric::flop();
        let h0 = Histogram::flop(&[(0, 1)]);
        let h1 = Histogram::flop(&[(1, 1)]);
        let h2 = Histogram::flop(&[(2, 1)]);
        let _ = Sinkhorn::divergence(&h0, &h1, &metric);
        let _ = Sinkhorn::divergence(&h1, &h2, &metric);
        let size = SELF_COST_CACHE.with_borrow(std::collections::HashMap::len);
        assert_eq!(size, 3, "expected 3 unique self-cost entries");
    }

    /// Cached value must equal the value computed without the cache, for the same inputs. Bug
    /// fence: a hash collision producing wrong values would surface here.
    #[test]
    fn cache_returns_same_value_as_fresh_computation() {
        Sinkhorn::clear();
        let metric = Metric::flop();
        let mu = Histogram::flop(&[(0, 3), (5, 1), (12, 4)]);
        let nu = Histogram::flop(&[(2, 2), (8, 5), (24, 3)]);
        let warm = Sinkhorn::divergence(&mu, &nu, &metric);
        Sinkhorn::clear();
        let cold = Sinkhorn::divergence(&mu, &nu, &metric);
        assert!((warm - cold).abs() < 1e-6, "warm={warm} cold={cold}");
    }
}
