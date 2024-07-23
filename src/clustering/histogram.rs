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
    pub fn centroid(histograms: Vec<&Histogram>) -> Histogram {
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
pub struct Centroid(Histogram, Vec<Histogram>);

impl Centroid {
    pub fn histogram(&self) -> &Histogram {
        &self.0
    }
    pub fn signature(&self) -> Abstraction {
        // could precompute if this BTreeMap Hash is too slow, but haven't profiled yet
        Abstraction::from(&self.0)
    }
    pub fn collect(&mut self, histogram: Histogram) {
        self.1.push(histogram);
    }
    pub fn collapse(&mut self) {
        self.0.sum = 0;
        self.0.weights.clear();
        for histogram in self.1.iter() {
            for (key, count) in histogram.weights.iter() {
                *self.0.weights.entry(*key).or_insert(0) += count;
            }
            self.0.sum += histogram.sum;
        }
        self.1.clear();
    }
}
