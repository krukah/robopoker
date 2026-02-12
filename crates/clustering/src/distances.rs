use super::*;
use rbp_cards::*;
use rbp_core::*;

/// Triangular array sizes: K*(K-1)/2 for each street's cluster count.
/// These give the number of unique unordered pairs of abstractions.
pub const TRI_PREF: usize = Street::Pref.n_abstractions() * (Street::Pref.n_abstractions() - 1) / 2;
pub const TRI_FLOP: usize = Street::Flop.n_abstractions() * (Street::Flop.n_abstractions() - 1) / 2;
pub const TRI_TURN: usize = Street::Turn.n_abstractions() * (Street::Turn.n_abstractions() - 1) / 2;

/// Dense triangular storage for pairwise distances between abstractions.
///
/// Stores the lower triangle of the symmetric distance matrix (excluding diagonal)
/// as a flat array indexed by [`Pair::triangular()`]. This halves memory usage
/// compared to a full matrix while enabling O(1) lookup.
///
/// # Const Generic
///
/// `T` is the triangular number K*(K-1)/2 where K is the street's abstraction count.
/// This enables stack allocation with compile-time size checking.
#[derive(Clone, Copy)]
pub struct Distances<const T: usize> {
    /// Street these distances are for.
    street: Street,
    /// Flat array of distance values indexed by triangular index.
    values: [Energy; T],
}

pub type DistPref = Distances<TRI_PREF>;
pub type DistFlop = Distances<TRI_FLOP>;
pub type DistTurn = Distances<TRI_TURN>;

impl<const T: usize> Distances<T> {
    /// Creates empty distance storage for the given street.
    pub const fn new(street: Street) -> Self {
        Self {
            street,
            values: [0.0; T],
        }
    }
    /// The street these distances are for.
    pub fn street(&self) -> Street {
        self.street
    }
    /// Gets distance for an abstraction pair.
    pub fn get(&self, pair: Pair) -> Energy {
        unsafe { *self.values.get_unchecked(pair.triangular()) }
    }
    /// Sets distance for an abstraction pair.
    pub fn set(&mut self, pair: Pair, value: Energy) {
        unsafe { *self.values.get_unchecked_mut(pair.triangular()) = value }
    }
    /// Iterates over (packed_pair, distance) pairs for database streaming.
    pub fn iter(&self) -> impl Iterator<Item = (i32, Energy)> + '_ {
        self.values
            .iter()
            .enumerate()
            .map(|(t, &d)| (Pair::split(t), d))
            .map(|((i, j), d)| (Pair::new(self.street(), i, j), d))
            .map(|(pair, dist)| (i32::from(pair), dist))
    }
    /// Normalizes all distances to [0, 1] by dividing by the maximum.
    pub fn normalize(&mut self) {
        let max = self
            .values
            .iter()
            .copied()
            .fold(f32::MIN_POSITIVE, f32::max);
        self.values.iter_mut().for_each(|v| *v /= max);
    }
}

impl<const T: usize> IntoIterator for Distances<T> {
    type Item = (i32, Energy);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + Send>;
    fn into_iter(self) -> Self::IntoIter {
        let street = self.street;
        Box::new(
            self.values
                .into_iter()
                .enumerate()
                .map(move |(t, d)| (Pair::split(t), d))
                .map(move |((i, j), d)| (Pair::new(street, i, j), d))
                .map(|(pair, dist)| (i32::from(pair), dist)),
        )
    }
}
