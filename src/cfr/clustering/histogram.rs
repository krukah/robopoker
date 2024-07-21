use super::abstraction::Abstraction;
use std::collections::BTreeMap;
use std::hash::Hash;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
/// We derive Hash from BTreeMap which allows us to identify a unique Histogram.
#[derive(Debug, Hash)]
pub struct Histogram {
    sum: usize,
    weights: BTreeMap<Abstraction, usize>,
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
    pub fn centroid(histograms: Vec<&Self>) -> Self {
        let mut centroid = Self::from(vec![]);
        for histogram in histograms {
            for (key, count) in histogram.weights.iter() {
                *centroid.weights.entry(*key).or_insert(0) += count;
            }
            centroid.sum += histogram.sum;
        }
        centroid
    }
}
