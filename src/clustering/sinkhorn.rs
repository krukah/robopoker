use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::metric::Metric;
use super::potential::Potential;
use crate::transport::coupling::Coupling;
use crate::transport::density::Density;
use crate::transport::measure::Measure;
use crate::Energy;
use crate::Entropy;
use std::collections::BTreeMap;

/// using this to represent an arbitrary instance of the Kontorovich-Rubinstein
/// potential formulation of the optimal transport problem.
pub struct Sinkhorn<'a> {
    metric: &'a Metric,
    mu: &'a Histogram,
    nu: &'a Histogram,
    lhs: Potential,
    rhs: Potential,
}

impl Sinkhorn<'_> {
    /// calculate ε-minimizing coupling by scaling potentials
    fn sinkhorn(&mut self) {
        for _ in 0..self.iterations() {
            let ref mut next = self.lhs();
            let ref mut prev = self.lhs;
            let lhs = Self::error(prev, next);
            std::mem::swap(prev, next);
            let ref mut next = self.rhs();
            let ref mut prev = self.rhs;
            let rhs = Self::error(prev, next);
            std::mem::swap(prev, next);
            if (lhs + rhs) < self.tolerance() {
                return;
            }
        }
        println!(
            "sinkhorn failed to converge in {} iterations",
            self.iterations()
        );
    }
    /// calculate next iteration of LHS and RHS potentials after Sinkhorn scaling
    fn lhs(&self) -> Potential {
        Potential::from(
            self.lhs
                .support()
                .copied()
                .map(|x| (x, self.divergence(&x, &self.mu, &self.rhs)))
                .inspect(|x| assert!(x.1.is_finite(), "lhs entropy overflow"))
                .collect::<BTreeMap<_, _>>(),
        )
    }
    /// calculate next iteration of LHS and RHS potentials after Sinkhorn scaling
    fn rhs(&self) -> Potential {
        Potential::from(
            self.rhs
                .support()
                .copied()
                .map(|x| (x, self.divergence(&x, &self.nu, &self.lhs)))
                .inspect(|x| assert!(x.1.is_finite(), "rhs entropy overflow"))
                .collect::<BTreeMap<_, _>>(),
        )
    }
    /// the coupling formed by joint distribution of LHS and RHS potentials
    fn coupling(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        (self.lhs.density(x) + self.rhs.density(y) - self.regularization(x, y)).exp()
    }
    /// update the potential energy on a given side
    /// histogram is where a: Abstraction is supported
    /// potential is the distribution that is being integrated against
    /// so we scale PDF(A::histogram | t) by the mass of the PDF(B::potential | t, x == a)
    /// not sure yet why i'm calling it entropy but it's giving partition function
    /// actually now that i think of it this might be KL div / relative entropy
    /// it might not be though
    fn divergence(&self, x: &Abstraction, histogram: &Histogram, potential: &Potential) -> Entropy {
        histogram.density(x).ln()
            - potential
                .support()
                .map(|y| potential.density(y) - self.regularization(x, y))
                .map(|e| e.exp())
                .map(|e| e.max(Energy::MIN_POSITIVE))
                .sum::<Energy>()
                .ln()
    }
    /// distance in fixed temperature exponent space
    fn regularization(&self, x: &Abstraction, y: &Abstraction) -> Entropy {
        self.metric.distance(x, y) / self.temperature()
    }
    /// stopping criteria
    fn error(last: &Potential, next: &Potential) -> Energy {
        next.support()
            .map(|x| next.density(x).exp() - last.density(x).exp())
            .map(|e| e.abs())
            .fold(0f32, f32::max)
    }
    /// hyperparameter that determines strength of entropic regularization. incorrect units but whatever
    const fn temperature(&self) -> Entropy {
        crate::SINKHORN_TEMPERATURE
    }
    /// hyperparameter that determines maximum number of iterations
    const fn iterations(&self) -> usize {
        crate::SINKHORN_ITERATIONS
    }
    /// hyperparameter that determines stopping criteria
    const fn tolerance(&self) -> Energy {
        crate::SINKHORN_TOLERANCE
    }
}

impl Coupling for Sinkhorn<'_> {
    type X = Abstraction;
    type Y = Abstraction;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn minimize(mut self) -> Self {
        self.sinkhorn();
        self
    }
    fn flow(&self, x: &Self::X, y: &Self::Y) -> Energy {
        self.coupling(x, y) * self.metric.distance(x, y)
    }
    fn cost(&self) -> Energy {
        self.lhs
            .support()
            .flat_map(|x| self.rhs.support().map(move |y| (x, y)))
            .map(|(x, y)| self.flow(x, y))
            .inspect(|x| assert!(x.is_finite()))
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
