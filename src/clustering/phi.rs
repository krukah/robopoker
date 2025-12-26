use super::*;
use crate::cards::*;
use crate::gameplay::*;
use crate::*;
use std::ops::AddAssign;

/// Sentinel value indicating an abstraction is not in the support.
const ABSENT: Entropy = Entropy::NEG_INFINITY;

/// Non-allocating potential over Abstractions.
/// Stores entropy values as a dense array indexed by abstraction index.
/// Uses NEG_INFINITY as sentinel for absent entries.
#[derive(Debug, Clone, Copy)]
pub struct Phi<const N: usize>([Entropy; N]);

impl<const N: usize> Phi<N> {
    pub const fn empty() -> Self {
        Self([ABSENT; N])
    }
    pub fn zeroes(bins: &Bins<N>) -> Self {
        let mut phi = Self::empty();
        bins.support().for_each(|a| phi.0[a.index()] = 0.);
        phi
    }
    pub fn uniform(bins: &Bins<N>) -> Self {
        let mut phi = Self::empty();
        let v = (1. / bins.n() as Probability).ln();
        bins.support().for_each(|a| phi.0[a.index()] = v);
        phi
    }
    pub fn derive(bins: &Bins<N>) -> Self {
        let mut phi = Self::empty();
        bins.support()
            .for_each(|a| phi.0[a.index()] = bins.density(&a));
        phi
    }
    pub fn density(&self, x: &Abstraction) -> Entropy {
        self.0[x.index()]
    }
    pub fn support(&self, street: Street) -> impl Iterator<Item = Abstraction> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, v)| v != &&ABSENT)
            .map(move |(i, _)| Abstraction::from((street, i)))
    }
    pub fn increment(&mut self, x: &Abstraction, delta: Entropy) {
        self.0[x.index()].add_assign(delta);
    }
    pub fn set(&mut self, x: &Abstraction, value: Entropy) {
        self.0[x.index()] = value;
    }
    pub fn values(&self) -> impl Iterator<Item = Entropy> + '_ {
        self.0.iter().copied().filter(|&v| v != ABSENT)
    }
}
