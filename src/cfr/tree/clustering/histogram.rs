use super::abstraction::Abstraction;
use std::collections::BTreeMap;
use std::hash::Hash;

#[derive(Debug, Hash)]
pub struct Histogram {
    n: usize,
    counts: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, x: &Abstraction) -> f32 {
        self.counts.get(x).cloned().unwrap_or(0) as f32 / self.n as f32
    }
    pub fn domain(&self) -> Vec<&Abstraction> {
        self.counts.keys().collect()
    }
    pub fn size(&self) -> usize {
        self.counts.len()
    }
    pub fn centroid(histograms: Vec<&Self>) -> Self {
        let mut centroid = Self::from(vec![]);
        for histogram in histograms {
            for (key, count) in histogram.counts.iter() {
                *centroid.counts.entry(*key).or_insert(0) += count;
            }
            centroid.n += histogram.n;
        }
        centroid
    }
}

impl From<Vec<Abstraction>> for Histogram {
    fn from(abstractions: Vec<Abstraction>) -> Self {
        let n = abstractions.len();
        let mut counts = BTreeMap::new();
        for abs in abstractions {
            *counts.entry(abs).or_insert(0usize) += 1;
        }
        Self { n, counts }
    }
}
