use crate::clustering::abstraction::Abstraction;
use std::collections::BTreeMap;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
#[derive(Debug, Default, Clone)]
pub struct Histogram {
    norm: usize,
    weights: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, abstraction: &Abstraction) -> f32 {
        self.weights.get(abstraction).copied().unwrap_or(0) as f32 / self.norm as f32
    }
    pub fn domain(&self) -> Vec<&Abstraction> {
        self.weights.keys().collect()
    }
    pub fn witness(self, abstraction: Abstraction) -> Self {
        let mut this = self;
        *this.weights.entry(abstraction).or_insert(0) += 1;
        this.norm += 1;
        this
    }
    /// Absorb the other histogram into this one.
    /// Note that this implicitly assumes sum normalizations are the same,
    /// which should hold until we implement Observation isomorphisms!
    pub fn absorb(&mut self, other: &Self) {
        self.norm += other.norm;
        for (key, count) in other.weights.iter() {
            *self.weights.entry(key.clone()).or_insert(0) += count;
        }
    }
}
