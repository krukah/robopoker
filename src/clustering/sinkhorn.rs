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
    /// hyperparameter that determines strength of entropic regularization. incorrect units but whatever
    const fn temperature(&self) -> Entropy {
        crate::SINKHORN_TEMPERATURE
    }
    /// hyperparameter that determines maximum number of iterations
    const fn iterations(&self) -> usize {
        crate::SINKHORN_ITERATIONS
    }
    #[allow(dead_code)]
    /// hyperparameter that determines stopping criteria
    const fn tolerance(&self) -> Energy {
        crate::SINKHORN_TOLERANCE
    }
    #[allow(dead_code)]
    /// stopping criteria
    fn delta(last: &Potential, next: &Potential) -> Energy {
        next.support()
            .map(|x| next.density(x).exp() - last.density(x).exp())
            .map(|x| x.abs())
            .fold(0f32, f32::max)
    }
    /// calculate ε-minimizing coupling by scaling potentials
    fn evolve(mut self) -> Self {
        for _ in 0..self.iterations() {
            self.lhs = self.lhs();
            self.rhs = self.rhs();
            // let ref mut next = self.lhs();
            // let lhs_delta = self.delta(&self.lhs, &next);
            // std::mem::swap(&mut self.lhs, next);
            // let ref mut next = self.rhs();
            // let rhs_delta = self.delta(&self.rhs, &next);
            // std::mem::swap(&mut self.rhs, next);
            // if (lhs_delta + rhs_delta) < self.tolerance() {
            //     // println!("converged in {} iterations", i);
            //     break;
            // }
        }
        self
    }
    /// calculate next iteration of LHS and RHS potentials after Sinkhorn scaling
    fn lhs(&self) -> Potential {
        Potential::from(
            self.lhs
                .support()
                .copied()
                .map(|x| (x, self.entropy(&x, &self.mu, &self.rhs)))
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
                .map(|x| (x, self.entropy(&x, &self.nu, &self.lhs)))
                .inspect(|x| assert!(x.1.is_finite(), "rhs entropy overflow"))
                .collect::<BTreeMap<_, _>>(),
        )
    }
    /// update the potential energy on a given side
    /// histogram is where a: Abstraction is supported
    /// potential is the distribution that is being integrated against
    /// so we scale PDF(A::histogram | t) by the mass of the PDF(B::potential | t, x == a)
    /// not sure yet why i'm calling it entropy but it's giving partition function
    /// actually now that i think of it this might be KL div / relative entropy
    /// it might not be though
    fn entropy(&self, a: &Abstraction, histogram: &Histogram, potential: &Potential) -> Entropy {
        histogram.density(a).ln()
            - potential
                .support()
                .map(|b| potential.density(b) - self.kernel(a, b))
                .map(|x| x.exp())
                .map(|x| x.max(Energy::MIN_POSITIVE))
                .sum::<Energy>()
                .ln()
    }
    /// the energy contributed by a given x, y Abstraction pair,
    /// using our scaled Potentials + regularizing kernel.
    fn energy(&self, x: &Abstraction, y: &Abstraction) -> Entropy {
        self.metric.distance(x, y) * self.boltzmann(x, y).exp()
    }
    /// the regularizing kernel
    fn kernel(&self, x: &Abstraction, y: &Abstraction) -> Entropy {
        self.metric.distance(x, y) / self.temperature()
    }
    fn boltzmann(&self, x: &Abstraction, y: &Abstraction) -> Entropy {
        self.lhs.density(x) + self.rhs.density(y) - self.kernel(x, y)
    }
}

impl Coupling for Sinkhorn<'_> {
    type X = Abstraction;
    type Y = Abstraction;
    type P = Potential;
    type Q = Potential;
    type M = Metric;

    fn minimize(self) -> Self {
        self.evolve()
    }
    fn flow(&self, x: &Self::X, y: &Self::Y) -> Entropy {
        self.boltzmann(x, y)
    }
    fn cost(&self) -> Energy {
        self.lhs
            .support()
            .flat_map(|x| self.rhs.support().map(move |y| (x, y)))
            .map(|(x, y)| self.energy(x, y))
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
