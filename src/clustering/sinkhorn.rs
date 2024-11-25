use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::metric::Metric;
use super::potential::Potential;
use crate::transport::coupling::Coupling;
use crate::transport::density::Density;
use crate::transport::measure::Measure;
use crate::Energy;
use crate::Probability;
use crate::Utility;
use std::collections::BTreeMap;
use std::ops::Neg;

enum Side {
    LHS,
    RHS,
}

/// using this to represent an arbitrary instance of the Kontorovich-Rubinstein
/// potential formulation of the optimal transport problem.
pub struct Sinkhorn<'a> {
    metric: &'a Metric,
    mu: &'a Histogram,
    nu: &'a Histogram,
    lhs: Potential,
    rhs: Potential,
    mass: Probability,
}

impl Sinkhorn<'_> {
    /// calculate ε-minimizing coupling by scaling potentials
    fn minimize(mut self) -> Self {
        for _ in 0..self.iterations() {
            self.lhs = self.lhs();
            self.rhs = self.rhs();
        }
        self
    }
    /// calculate next iteration of LHS and RHS potentials after Sinkhorn scaling
    fn lhs(&self) -> Potential {
        self.lhs
            .support()
            .copied()
            .map(|x| (x, self.energy(&x, self.mu, &self.rhs)))
            .collect::<BTreeMap<_, _>>()
            .into()
    }
    /// calculate next iteration of LHS and RHS potentials after Sinkhorn scaling
    fn rhs(&self) -> Potential {
        self.rhs
            .support()
            .copied()
            .map(|x| (x, self.energy(&x, self.nu, &self.lhs)))
            .collect::<BTreeMap<_, _>>()
            .into()
    }
    /// update the potential energy on a given side
    fn energy(&self, a: &Abstraction, histogram: &Histogram, potential: &Potential) -> Energy {
        histogram.density(a).ln()
            - potential
                .support()
                .map(|b| potential.density(b) - self.kernel(a, b))
                .map(|x| x.exp())
                .sum::<Energy>()
                .ln()
    }
    /// compute frobenius norm of the coupling w.r.t. given metric
    fn frobenius(&self) -> Energy {
        self.lhs
            .support()
            .flat_map(|x| self.rhs.support().map(move |y| (x, y)))
            .map(|(x, y)| self.flow(x, y) * self.metric.distance(x, y))
            .inspect(|x| assert!(!x.is_nan()))
            .inspect(|x| assert!(x.is_finite()))
            .sum::<Energy>()
    }

    fn flow(&self, x: &Abstraction, y: &Abstraction) -> Probability {
        (self.lhs.density(x) + self.rhs.density(y) - self.kernel(x, y)).exp()
    }
    fn kernel(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        self.metric.distance(x, y) / self.epsilon()
    }
    fn mass(&self) -> Energy {
        self.mass
    }

    /// hyperparameter that determines maximum number of iterations
    const fn iterations(&self) -> usize {
        100
    }
    /// hyperparameter that determines strength of entropic regularization
    const fn epsilon(&self) -> Energy {
        1e-2
    }
}

impl Coupling for Sinkhorn<'_> {
    type X = Abstraction;
    type Y = Abstraction;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn minimize(self) -> Self {
        self.minimize()
    }
    fn flow(&self, x: &Self::X, y: &Self::Y) -> Utility {
        self.flow(x, y)
    }
    fn cost(&self) -> Utility {
        self.frobenius()
    }
}

impl<'a> From<(&'a Histogram, &'a Histogram, &'a Metric)> for Sinkhorn<'a> {
    fn from((p, q, metric): (&'a Histogram, &'a Histogram, &'a Metric)) -> Self {
        Self {
            mass: 1.,
            metric,
            mu: p,
            nu: q,
            lhs: p.uniform(),
            rhs: q.uniform(),
        }
    }
}
