use super::equivalence::Abstraction;
use std::collections::BTreeMap;
use std::hash::Hash;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
/// We derive Hash from BTreeMap which allows us to identify a unique Histogram.
#[derive(Debug, Hash, Default)]
pub struct Histogram {
    pub(super) sum: usize,
    pub(super) weights: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, abstraction: &Abstraction) -> f32 {
        self.weights.get(abstraction).copied().unwrap_or(0) as f32 / self.sum as f32
    }
    pub fn domain(&self) -> Vec<&Abstraction> {
        self.weights.keys().collect()
    }
    pub fn size(&self) -> usize {
        self.weights.len()
    }
    pub fn merge(&mut self, other: &Self) {
        self.sum += other.sum;
        for (key, count) in other.weights.iter() {
            *self.weights.entry(key.clone()).or_insert(0) += count;
        }
    }
}

impl From<Vec<Abstraction>> for Histogram {
    fn from(abstractions: Vec<Abstraction>) -> Self {
        let sum = abstractions.len();
        let mut weights = BTreeMap::new();
        for abs in abstractions {
            *weights.entry(abs).or_insert(0usize) += 1;
        }
        Self { sum, weights }
    }
}

/// A Centroid is a collection of Histograms.
///
/// It is used to collect histograms and collapse them into a single histogram.
/// Tightly coupled with k-means implementaiton in Layer
pub struct Centroid(Histogram, Abstraction);

impl Centroid {
    pub fn histogram(&self) -> &Histogram {
        &self.0
    }
    pub fn signature(&self) -> Abstraction {
        // could precompute if this BTreeMap Hash is too slow, but haven't profiled yet
        Abstraction::from(&self.0)
    }
    /// maybe we don't keep the new Histogram in memory, and keep a running average to preserve meory
    pub fn merge(&mut self, other: &Histogram) {
        self.0.merge(other);
    }
}
