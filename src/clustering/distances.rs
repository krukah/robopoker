use super::*;
use crate::cards::*;
use crate::*;

/// Triangular array sizes: K*(K-1)/2 for each street's cluster count
pub const TRI_PREF: usize = Street::Pref.n_abstractions() * (Street::Pref.n_abstractions() - 1) / 2;
pub const TRI_FLOP: usize = Street::Flop.n_abstractions() * (Street::Flop.n_abstractions() - 1) / 2;
pub const TRI_TURN: usize = Street::Turn.n_abstractions() * (Street::Turn.n_abstractions() - 1) / 2;

/// Fixed-size distance storage for a specific street's abstractions.
#[derive(Clone, Copy)]
pub struct Distances<const T: usize> {
    street: Street,
    values: [Energy; T],
}

pub type DistPref = Distances<TRI_PREF>;
pub type DistFlop = Distances<TRI_FLOP>;
pub type DistTurn = Distances<TRI_TURN>;

impl<const T: usize> Distances<T> {
    pub const fn new(street: Street) -> Self {
        Self {
            street,
            values: [0.0; T],
        }
    }
    pub fn street(&self) -> Street {
        self.street
    }
    pub fn get(&self, pair: Pair) -> Energy {
        self.values[pair.triangular()]
    }
    pub fn set(&mut self, pair: Pair, value: Energy) {
        self.values[pair.triangular()] = value;
    }
    pub fn iter(&self) -> impl Iterator<Item = (i32, Energy)> + '_ {
        self.values
            .iter()
            .enumerate()
            .map(|(t, &d)| (Pair::split(t), d))
            .map(|((i, j), d)| (Pair::new(self.street(), i, j), d))
            .map(|(pair, dist)| (i32::from(pair), dist))
    }
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
