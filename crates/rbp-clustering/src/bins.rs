use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use std::ops::AddAssign;

/// Street-specific histogram sizes derived from clustering parameters.
/// Each value is `Street::n_abstractions()` for that street.
pub const N_PREF: usize = Street::Pref.n_abstractions();
pub const N_FLOP: usize = Street::Flop.n_abstractions();
pub const N_TURN: usize = Street::Turn.n_abstractions();
pub const N_RIVE: usize = Street::Rive.n_abstractions();

/// Type aliases for street-specific bin arrays.
pub type BinsPref = Bins<N_PREF>;
pub type BinsFlop = Bins<N_FLOP>;
pub type BinsTurn = Bins<N_TURN>;
pub type BinsRive = Bins<N_RIVE>;

/// A zero-allocation distribution over abstraction buckets.
///
/// Stores counts as a dense array indexed by [`Abstraction::index()`].
/// The `weight` field tracks total mass for efficient density computation.
///
/// # Const Generics
///
/// The array size `N` is determined at compile time based on the street's
/// abstraction count. This enables stack allocation while supporting
/// different sizes per street (e.g., 169 preflop vs 200 flop buckets).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bins<const N: usize> {
    /// The street these bins represent.
    street: Street,
    /// Total count across all bins (normalization denominator).
    weight: usize,
    /// Dense array of counts indexed by abstraction index.
    counts: [usize; N],
}

impl<const N: usize> Bins<N> {
    /// Creates an empty bin array for the given street.
    pub const fn new(street: Street) -> Self {
        Self {
            street,
            weight: 0,
            counts: [0; N],
        }
    }
    /// Sets the count for a specific abstraction.
    pub fn set(&mut self, abs: Abstraction, count: usize) {
        unsafe { *self.counts.get_unchecked_mut(abs.index()) = count }
        self.weight += count;
    }
    /// Number of non-zero bins (support size).
    pub fn n(&self) -> usize {
        self.counts.iter().filter(|&&c| c > 0).count()
    }
    /// Probability mass at a specific abstraction.
    pub fn density(&self, x: &Abstraction) -> Probability {
        unsafe { *self.counts.get_unchecked(x.index()) as f32 / self.weight as f32 }
    }
    /// Iterates over (index, count) pairs.
    pub fn counts(&self) -> impl Iterator<Item = (usize, &usize)> + '_ {
        self.counts.iter().enumerate()
    }
    /// The street these bins represent.
    pub fn street(&self) -> Street {
        self.street
    }
    /// Increments the count for an abstraction by 1.
    pub fn increment(&mut self, abs: Abstraction) {
        self.weight.add_assign(1usize);
        unsafe {
            self.counts
                .get_unchecked_mut(abs.index())
                .add_assign(1usize)
        }
    }
    /// Merges another bin array into this one.
    pub fn merge<const M: usize>(&mut self, other: &Bins<M>) {
        debug_assert!(N == M);
        self.weight += other.weight;
        self.counts
            .iter_mut()
            .zip(other.counts)
            .for_each(|(a, b)| a.add_assign(b));
    }
    /// Iterates over abstractions with non-zero counts.
    pub fn support(&self) -> impl Iterator<Item = Abstraction> + '_ {
        self.counts()
            .filter(|&(_, &count)| count > 0)
            .map(|(i, _)| Abstraction::from((self.street(), i)))
    }
    /// Returns first abstraction in support (for type inference).
    pub fn peek(&self) -> Abstraction {
        self.support().next().expect("non empty histogram")
    }
    /// Computes expected equity for river histograms.
    /// Only valid when street is River with Equity abstractions.
    pub fn equity(&self) -> Probability {
        debug_assert!(matches!(self.street(), Street::Rive));
        debug_assert!(matches!(self.peek().street(), Street::Rive));
        self.pdf().iter().map(|(x, y)| x * y).sum()
    }
    /// Returns (equity, probability) pairs for visualization.
    /// The equity abstraction is converted to its [0,1] value.
    pub fn pdf(&self) -> Vec<(Probability, Probability)> {
        debug_assert!(matches!(self.street(), Street::Rive));
        debug_assert!(matches!(self.peek().street(), Street::Rive));
        self.counts()
            .filter(|(_, c)| c > &&0)
            .map(|(i, &count)| (Abstraction::from((self.street(), i)), count as f32))
            .map(|(a, b)| (a, b / self.weight as f32))
            .map(|(k, v)| (Probability::from(k), Probability::from(v)))
            .collect()
    }
    /// Returns (abstraction, density) pairs sorted by density descending.
    pub fn distribution(&self) -> Vec<(Abstraction, Probability)> {
        let mut distribution = self
            .support()
            .map(|abs| (abs, self.density(&abs)))
            .collect::<Vec<_>>();
        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        distribution
    }
}
