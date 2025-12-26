use crate::cards::*;
use crate::gameplay::*;
use crate::*;
use std::ops::AddAssign;

/// Street-specific histogram sizes derived from clustering parameters.
pub const N_PREF: usize = Street::Pref.n_abstractions();
pub const N_FLOP: usize = Street::Flop.n_abstractions();
pub const N_TURN: usize = Street::Turn.n_abstractions();
pub const N_RIVE: usize = Street::Rive.n_abstractions();

/// Type aliases for street-specific histograms.
pub type BinsPref = Bins<N_PREF>;
pub type BinsFlop = Bins<N_FLOP>;
pub type BinsTurn = Bins<N_TURN>;
pub type BinsRive = Bins<N_RIVE>;

/// A non-allocating distribution over Abstractions for a specific street.
/// Stores counts as a dense array indexed by abstraction index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bins<const N: usize> {
    street: Street,
    weight: usize,
    counts: [usize; N],
}

impl<const N: usize> Bins<N> {
    pub const fn new(street: Street) -> Self {
        Self {
            street,
            weight: 0,
            counts: [0; N],
        }
    }
    pub fn set(&mut self, abs: Abstraction, count: usize) {
        self.counts[abs.index()] = count;
        self.weight += count;
    }
    pub fn n(&self) -> usize {
        self.counts.iter().filter(|&&c| c > 0).count()
    }
    pub fn density(&self, x: &Abstraction) -> Probability {
        self.counts[x.index()] as f32 / self.weight as f32
    }
    pub fn counts(&self) -> impl Iterator<Item = (usize, &usize)> + '_ {
        self.counts.iter().enumerate()
    }
    pub fn street(&self) -> Street {
        self.street
    }
    pub fn increment(&mut self, abs: Abstraction) {
        self.weight.add_assign(1usize);
        self.counts[abs.index()].add_assign(1usize);
    }
    pub fn merge<const M: usize>(&mut self, other: &Bins<M>) {
        assert!(N == M);
        self.weight += other.weight;
        self.counts
            .iter_mut()
            .zip(other.counts)
            .for_each(|(a, b)| a.add_assign(b));
    }
    pub fn support(&self) -> impl Iterator<Item = Abstraction> + '_ {
        self.counts()
            .filter(|&(_, &count)| count > 0)
            .map(|(i, _)| Abstraction::from((self.street(), i)))
    }
    pub fn peek(&self) -> Abstraction {
        self.support().next().expect("non empty histogram")
    }
    pub fn equity(&self) -> Probability {
        assert!(matches!(self.street(), Street::Rive));
        assert!(matches!(self.peek().street(), Street::Rive));
        self.pdf().iter().map(|(x, y)| x * y).sum()
    }
    pub fn pdf(&self) -> Vec<(Probability, Probability)> {
        assert!(matches!(self.street(), Street::Rive));
        assert!(matches!(self.peek().street(), Street::Rive));
        self.counts()
            .filter(|(_, c)| c > &&0)
            .map(|(i, &count)| (Abstraction::from((self.street(), i)), count as f32))
            .map(|(a, b)| (a, b / self.weight as f32))
            .map(|(k, v)| (Probability::from(k), Probability::from(v)))
            .collect()
    }
    pub fn distribution(&self) -> Vec<(Abstraction, Probability)> {
        let mut distribution = self
            .support()
            .map(|abs| (abs, self.density(&abs)))
            .collect::<Vec<_>>();
        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        distribution
    }
}
