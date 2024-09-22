use crate::clustering::abstraction::NodeAbstraction;
use std::collections::HashMap;
use std::ops::AddAssign;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
#[derive(Debug, Default, Clone)]
pub struct Histogram {
    norm: usize,
    weights: HashMap<NodeAbstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, abstraction: &NodeAbstraction) -> f32 {
        self.weights.get(abstraction).copied().unwrap_or(0usize) as f32 / self.norm as f32
    }
    pub fn domain(&self) -> Vec<&NodeAbstraction> {
        self.weights.keys().collect()
    }
    pub fn witness(self, abstraction: NodeAbstraction) -> Self {
        let mut this = self;
        this.norm.add_assign(1usize);
        this.weights
            .entry(abstraction)
            .or_insert(0usize)
            .add_assign(1usize);
        this
    }
    pub fn clear(&mut self) {
        self.norm = 0;
        self.weights.clear();
    }
    /// Absorb the other histogram into this one.
    /// Note that this implicitly assumes sum normalizations are the same,
    /// which should hold until we implement Observation isomorphisms!
    pub fn absorb(&mut self, other: &Self) {
        self.norm += other.norm;
        for (key, count) in other.weights.iter() {
            self.weights
                .entry(key.to_owned())
                .or_insert(0usize)
                .add_assign(count.to_owned());
        }
    }
}
