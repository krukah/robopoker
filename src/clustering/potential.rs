use super::abstraction::Abstraction;
use super::histogram::Histogram;
use crate::transport::density::Density;
use crate::Entropy;
use crate::Probability;
use std::collections::BTreeMap;

/// using this to represent an arbitrary instance of the Kontorovich-Rubinstein
/// potential formulation of the optimal transport problem.
/// this structure can also be treated as a normalized distribution over Abstractions.
pub struct Potential(BTreeMap<Abstraction, Entropy>);

impl Potential {
    /// useful for Heuristic where we don't need to allocate.
    /// i guess we don't need to allocate in Sinkhorn either. but it's
    /// nbd, + we might want to calaculate deltas between new and old potentials
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Abstraction, &mut Entropy)> {
        self.0.iter_mut()
    }

    /// also only useful for Heuristic
    pub fn values(&self) -> impl Iterator<Item = &Entropy> {
        self.0.values()
    }

    /// uniform distribution over the support
    pub fn uniform(h: &Histogram) -> Self {
        Self(
            h.support()
                .copied()
                .map(|x| (x, h.n()))
                .map(|(x, y)| (x, 1. / y as Probability))
                .map(|(x, y)| (x, y.ln() as Entropy))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    /// unit normalized distribution over the support
    pub fn normalize(h: &Histogram) -> Self {
        Self(
            h.support()
                .copied()
                .map(|x| (x, h.density(&x)))
                .collect::<BTreeMap<_, _>>(),
        )
    }
}

impl From<BTreeMap<Abstraction, Entropy>> for Potential {
    fn from(potential: BTreeMap<Abstraction, Entropy>) -> Self {
        Self(potential)
    }
}

impl Density for Potential {
    type S = Abstraction;

    fn density(&self, x: &Self::S) -> Entropy {
        self.0
            .get(x)
            .copied()
            .inspect(|p| assert!(p.is_finite(), "density overflow"))
            .expect("abstraction in potential")
    }
    fn support(&self) -> impl Iterator<Item = &Self::S> {
        self.0.keys()
    }
}
