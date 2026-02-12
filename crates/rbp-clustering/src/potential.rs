use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;

/// Street-specific potential type aliases.
pub type PhiPref = Phi<N_PREF>;
pub type PhiFlop = Phi<N_FLOP>;
pub type PhiTurn = Phi<N_TURN>;
pub type PhiRive = Phi<N_RIVE>;

/// Dual potential for optimal transport computation.
///
/// In the Kantorovich-Rubinstein dual, the EMD equals the max over Lipschitz
/// potentials. Sinkhorn iteration finds these potentials via alternating
/// projection onto the marginal constraints.
///
/// # Stack Allocation
///
/// Like [`Histogram`], uses a tagged enum over fixed-size [`Phi`] arrays
/// to avoid heap allocation during EMD computation.
#[derive(Debug, Clone, Copy)]
pub enum Potential {
    Pref(PhiPref),
    Flop(PhiFlop),
    Turn(PhiTurn),
    Rive(PhiRive),
}

impl Potential {
    /// Creates a potential with zero values on the histogram's support.
    pub fn zeroes(h: &Histogram) -> Self {
        match h {
            Histogram::Pref(b) => Potential::Pref(Phi::zeroes(b)),
            Histogram::Flop(b) => Potential::Flop(Phi::zeroes(b)),
            Histogram::Turn(b) => Potential::Turn(Phi::zeroes(b)),
            Histogram::Rive(b) => Potential::Rive(Phi::zeroes(b)),
        }
    }
    /// Creates a uniform potential (log(1/n) at each support element).
    pub fn uniform(h: &Histogram) -> Self {
        match h {
            Histogram::Pref(b) => Potential::Pref(Phi::uniform(b)),
            Histogram::Flop(b) => Potential::Flop(Phi::uniform(b)),
            Histogram::Turn(b) => Potential::Turn(Phi::uniform(b)),
            Histogram::Rive(b) => Potential::Rive(Phi::uniform(b)),
        }
    }
    /// Creates a potential from histogram densities.
    pub fn derive(h: &Histogram) -> Self {
        match h {
            Histogram::Pref(b) => Potential::Pref(Phi::derive(b)),
            Histogram::Flop(b) => Potential::Flop(Phi::derive(b)),
            Histogram::Turn(b) => Potential::Turn(Phi::derive(b)),
            Histogram::Rive(b) => Potential::Rive(Phi::derive(b)),
        }
    }
    /// Gets potential value at an abstraction.
    pub fn density(&self, x: &Abstraction) -> Entropy {
        match self {
            Potential::Pref(p) => p.density(x),
            Potential::Flop(p) => p.density(x),
            Potential::Turn(p) => p.density(x),
            Potential::Rive(p) => p.density(x),
        }
    }
    /// Iterates over abstractions in the support.
    pub fn support(&self) -> impl Iterator<Item = Abstraction> + '_ {
        match self {
            Potential::Pref(p) => IterWrap::Pref(p.support(Street::Pref)),
            Potential::Flop(p) => IterWrap::Flop(p.support(Street::Flop)),
            Potential::Turn(p) => IterWrap::Turn(p.support(Street::Turn)),
            Potential::Rive(p) => IterWrap::Rive(p.support(Street::Rive)),
        }
    }
    /// Adds delta to potential value at x.
    pub fn increment(&mut self, x: &Abstraction, delta: Entropy) {
        match self {
            Potential::Pref(p) => p.increment(x, delta),
            Potential::Flop(p) => p.increment(x, delta),
            Potential::Turn(p) => p.increment(x, delta),
            Potential::Rive(p) => p.increment(x, delta),
        }
    }
    /// Sets potential value at x.
    pub fn set(&mut self, x: &Abstraction, value: Entropy) {
        match self {
            Potential::Pref(p) => p.set(x, value),
            Potential::Flop(p) => p.set(x, value),
            Potential::Turn(p) => p.set(x, value),
            Potential::Rive(p) => p.set(x, value),
        }
    }
    /// Iterates over non-absent potential values.
    pub fn values(&self) -> impl Iterator<Item = Entropy> + '_ {
        match self {
            Potential::Pref(p) => IterValWrap::Pref(p.values()),
            Potential::Flop(p) => IterValWrap::Flop(p.values()),
            Potential::Turn(p) => IterValWrap::Turn(p.values()),
            Potential::Rive(p) => IterValWrap::Rive(p.values()),
        }
    }
}

/// Helper enum to unify different support iterators
enum IterWrap<A, B, C, D> {
    Pref(A),
    Flop(B),
    Turn(C),
    Rive(D),
}

impl<A, B, C, D> Iterator for IterWrap<A, B, C, D>
where
    A: Iterator<Item = Abstraction>,
    B: Iterator<Item = Abstraction>,
    C: Iterator<Item = Abstraction>,
    D: Iterator<Item = Abstraction>,
{
    type Item = Abstraction;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterWrap::Pref(i) => i.next(),
            IterWrap::Flop(i) => i.next(),
            IterWrap::Turn(i) => i.next(),
            IterWrap::Rive(i) => i.next(),
        }
    }
}

/// Helper enum to unify different value iterators
enum IterValWrap<A, B, C, D> {
    Pref(A),
    Flop(B),
    Turn(C),
    Rive(D),
}

impl<A, B, C, D> Iterator for IterValWrap<A, B, C, D>
where
    A: Iterator<Item = Entropy>,
    B: Iterator<Item = Entropy>,
    C: Iterator<Item = Entropy>,
    D: Iterator<Item = Entropy>,
{
    type Item = Entropy;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterValWrap::Pref(i) => i.next(),
            IterValWrap::Flop(i) => i.next(),
            IterValWrap::Turn(i) => i.next(),
            IterValWrap::Rive(i) => i.next(),
        }
    }
}

impl Density for Potential {
    type Support = Abstraction;
    fn density(&self, x: &Self::Support) -> Entropy {
        self.density(x)
    }
    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.support()
    }
}
