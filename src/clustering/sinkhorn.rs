use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::metric::Metric;
use super::potential::Potential;
use crate::transport::coupling::Coupling;
use crate::transport::density::Density;
use crate::transport::measure::Measure;
use crate::Distance;
use crate::Probability;
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
    p: &'a Histogram,
    q: &'a Histogram,
    lhs: Potential,
    rhs: Potential,
    mass: Probability,
}

impl<'a> Sinkhorn<'a> {
    /// calculate ε-minimizing coupling by scaling potentials
    fn minimize(mut self) -> Self {
        for _ in 0..self.iterations() {
            self.lhs = self
                .lhs
                .support()
                .copied()
                .map(|x| (x, self.update(&x, Side::LHS)))
                .collect::<BTreeMap<_, _>>()
                .into();
            self.rhs = self
                .rhs
                .support()
                .copied()
                .map(|y| (y, self.update(&y, Side::RHS)))
                .collect::<BTreeMap<_, _>>()
                .into();
        }
        self
    }
    /// marginalize over the other side of the coupling
    fn update(&self, a: &Abstraction, side: Side) -> Probability {
        let (density, marginal) = match side {
            Side::LHS => (self.p.density(a), &self.rhs),
            Side::RHS => (self.q.density(a), &self.lhs),
        };
        density
            / marginal
                .support()
                .map(|b| marginal.density(b) * self.kernel(a, b))
                .sum::<Probability>()
    }

    /// compute frobenius norm of the coupling w.r.t. given metric
    fn frobenius(&self) -> Distance {
        self.lhs
            .support()
            .map(|x| self.rhs.support().map(move |y| (x, y)))
            .flatten()
            .map(|(x, y)| self.flow(x, y))
            .sum()
    }
    /// alternatively, could implemment Measure for Potential<'_>
    /// and interpret as living in a different entropically regularized metric
    /// space, but the intent is more clear this way probably.
    fn kernel(&self, x: &Abstraction, y: &Abstraction) -> Distance {
        (self.metric.distance(x, y) / self.epsilon() / self.mass())
            .neg()
            .exp()
    }
    /// compute local cost of coupling w.r.t. given metric
    fn flow(&self, x: &Abstraction, y: &Abstraction) -> Distance {
        self.lhs.density(x) * self.kernel(x, y) * self.rhs.density(y)
    }
    /// normalization of metric subspace supported by the coupling
    fn mass(&self) -> Distance {
        self.mass
    }

    /// hyperparameter that determines maximum number of iterations
    const fn iterations(&self) -> usize {
        10
    }
    /// hyperparameter that determines strength of entropic regularization
    const fn epsilon(&self) -> Distance {
        1e-4
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
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32 {
        self.flow(x, y)
    }
    fn cost(&self) -> f32 {
        self.frobenius()
    }
}

impl<'a> From<(&'a Histogram, &'a Histogram, &'a Metric)> for Sinkhorn<'a> {
    fn from((p, q, metric): (&'a Histogram, &'a Histogram, &'a Metric)) -> Self {
        Self {
            p,
            q,
            metric,
            lhs: p.uniformed(),
            rhs: q.uniformed(),
            mass: p
                .support()
                .map(|x| q.support().map(move |y| (x, y)))
                .flatten()
                .map(|(x, y)| metric.distance(&x, &y))
                .sum::<Distance>(),
        }
    }
}
