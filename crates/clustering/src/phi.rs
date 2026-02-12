use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use std::ops::AddAssign;

/// Sentinel value indicating an abstraction is not in the support.
const ABSENT: Entropy = Entropy::NEG_INFINITY;

/// Zero-allocation potential array for Sinkhorn iteration.
///
/// Stores entropy (log-probability) values indexed by abstraction index.
/// Uses `NEG_INFINITY` as a sentinel for absent entries, enabling
/// sparse support while maintaining dense storage.
///
/// # Const Generic
///
/// `N` is the street's abstraction count, enabling compile-time sizing.
#[derive(Debug, Clone, Copy)]
pub struct Phi<const N: usize>([Entropy; N]);

impl<const N: usize> Phi<N> {
    /// Creates an empty potential (all absent).
    pub const fn empty() -> Self {
        Self([ABSENT; N])
    }
    /// Creates a potential with zero values on support.
    pub fn zeroes(bins: &Bins<N>) -> Self {
        let mut phi = Self::empty();
        bins.support().for_each(|a| phi.set(&a, 0.));
        phi
    }
    /// Creates a uniform potential (log(1/n) at each element).
    pub fn uniform(bins: &Bins<N>) -> Self {
        let mut phi = Self::empty();
        let v = (1. / bins.n() as Probability).ln();
        bins.support().for_each(|a| phi.set(&a, v));
        phi
    }
    /// Creates a potential from bin densities.
    pub fn derive(bins: &Bins<N>) -> Self {
        let mut phi = Self::empty();
        bins.support().for_each(|a| phi.set(&a, bins.density(&a)));
        phi
    }
    /// Gets potential value at an abstraction.
    pub fn density(&self, x: &Abstraction) -> Entropy {
        unsafe { *self.0.get_unchecked(x.index()) }
    }
    /// Iterates over abstractions with non-absent values.
    pub fn support(&self, street: Street) -> impl Iterator<Item = Abstraction> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, v)| v != &&ABSENT)
            .map(move |(i, _)| Abstraction::from((street, i)))
    }
    /// Adds delta to potential value.
    pub fn increment(&mut self, x: &Abstraction, delta: Entropy) {
        unsafe { self.0.get_unchecked_mut(x.index()).add_assign(delta) }
    }
    /// Sets potential value.
    pub fn set(&mut self, x: &Abstraction, value: Entropy) {
        unsafe { *self.0.get_unchecked_mut(x.index()) = value }
    }
    /// Iterates over non-absent values.
    pub fn values(&self) -> impl Iterator<Item = Entropy> + '_ {
        self.0.iter().copied().filter(|&v| v != ABSENT)
    }
}
