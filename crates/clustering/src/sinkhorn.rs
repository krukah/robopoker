use super::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;

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
            .map(|x| (x, self.divergence(&x, self.mu, &self.rhs)))
            .inspect(|(_, d)| debug_assert!(d.is_finite(), "lhs entropy overflow"))
            .for_each(|(x, d)| next.set(&x, d));
        next
    }
    /// Computes updated RHS potential via Sinkhorn scaling.
    fn rhs(&self) -> Potential {
        let mut next = Potential::zeroes(self.nu);
        self.rhs
            .support()
            .map(|x| (x, self.divergence(&x, self.nu, &self.lhs)))
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
    fn divergence(&self, x: &Abstraction, histogram: &Histogram, potential: &Potential) -> Entropy {
        histogram.density(x).ln()
            - potential
                .support()
                .map(|y| potential.density(&y) - self.regularization(x, &y))
                .map(|e| e.exp())
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
            .map(|e| e.abs())
            .sum::<Energy>()
    }
    /// Entropic regularization strength. Lower = closer to exact EMD.
    const fn temperature(&self) -> Entropy {
        rbp_core::SINKHORN_TEMPERATURE
    }
    /// Maximum iteration count before forced termination.
    const fn iterations(&self) -> usize {
        rbp_core::SINKHORN_ITERATIONS
    }
    /// Convergence tolerance for early stopping.
    const fn tolerance(&self) -> Energy {
        rbp_core::SINKHORN_TOLERANCE
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
