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
    mu: &'a Histogram,
    nu: &'a Histogram,
    lhs: Potential,
    rhs: Potential,
    mass: Probability,
}

impl Sinkhorn<'_> {
    /// calculate ε-minimizing coupling by scaling potentials
    fn minimize(mut self) -> Self {
        for i in 0..self.iterations() {
            // println!("ITERATION {i}");
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
    /// update the potential on a given side
    fn update(&self, a: &Abstraction, side: Side) -> Probability {
        let (p_a, _, v) = match side {
            Side::LHS => (self.mu.density(a), 0, &self.rhs),
            Side::RHS => (self.nu.density(a), 0, &self.lhs),
        };
        let update = p_a.ln()
            - v.support()
                // .inspect(|b| println!("density {b} {} kernel {}", v.density(b), self.kernel(a, b)))
                .map(|b| v.density(b) * self.kernel(a, b))
                .sum::<Probability>()
                .ln();
        let update_exp = update.exp();
        assert!(update.is_finite(), "update overflow \n{update}");
        assert!(update_exp.is_finite(), "update.exp() overflow \n{update}",);
        update_exp
    }
    /// compute frobenius norm of the coupling w.r.t. given metric
    fn frobenius(&self) -> Distance {
        self.lhs
            .support()
            .flat_map(|x| self.rhs.support().map(move |y| (x, y)))
            // .inspect(|(x, y)| self.inspection(x, y))
            .map(|(x, y)| {
                self.kernel(x, y)
                    * self.metric.distance(x, y)
                    * self.lhs.density(x)
                    * self.rhs.density(y)
            })
            .inspect(|x| assert!(!x.is_nan()))
            .inspect(|x| assert!(x.is_finite()))
            .sum::<Distance>()
    }
    /// alternatively, could implemment Measure for Potential<'_>
    /// and interpret as living in a different entropically regularized metric
    /// space, but the intent is more clear this way probably.
    fn kernel(&self, x: &Abstraction, y: &Abstraction) -> Distance {
        (self.metric.distance(x, y) / self.epsilon() / self.mass())
            .neg()
            .exp()
    }
    /// normalization of metric subspace supported by the coupling
    fn mass(&self) -> Distance {
        self.mass
    }

    /// hyperparameter that determines maximum number of iterations
    const fn iterations(&self) -> usize {
        100
    }
    /// hyperparameter that determines strength of entropic regularization
    const fn epsilon(&self) -> Distance {
        1e-2
    }

    fn inspection(&self, x: &Abstraction, y: &Abstraction) {
        println!(
            "FLOW {x} {y} {:>6.2e} * {:>6.2e}, * {:>6.2e}",
            self.mu.density(x),
            self.nu.density(y),
            self.kernel(x, y)
        );
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
        self.kernel(x, y)
    }
    fn cost(&self) -> f32 {
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
