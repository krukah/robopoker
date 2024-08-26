use crate::clustering::abstraction::Abstraction;
use std::collections::BTreeMap;
use std::hash::Hash;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
/// We derive Hash from BTreeMap which allows us to identify a unique Histogram.
#[derive(Debug, Hash, Default, Clone)]
pub struct Histogram {
    sum: usize,
    weights: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, abstraction: &Abstraction) -> f32 {
        self.weights.get(abstraction).copied().unwrap_or(0) as f32 / self.sum as f32
    }
    pub fn domain(&self) -> Vec<&Abstraction> {
        self.weights.keys().collect()
    }
    pub fn absorb(&mut self, other: &Self) {
        self.sum += other.sum;
        for (key, count) in other.weights.iter() {
            *self.weights.entry(key.clone()).or_insert(0) += count;
        }
    }
    pub fn witness(self, abstraction: Abstraction) -> Self {
        let mut this = self;
        *this.weights.entry(abstraction).or_insert(0) += 1;
        this.sum += 1;
        this
    }
}
