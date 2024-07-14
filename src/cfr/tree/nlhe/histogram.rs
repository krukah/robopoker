use super::abstraction::Abstraction;
use std::collections::BTreeMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub struct Histogram {
    n: usize,
    counts: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, x: &Abstraction) -> f32 {
        *self.counts.get(x).unwrap_or(&0) as f32 / self.n as f32
    }
    pub fn domain(&self) -> Vec<&Abstraction> {
        self.counts.keys().collect()
    }
    pub fn size(&self) -> usize {
        self.counts.len()
    }
    pub fn abstraction(&self) -> Abstraction {
        let ref mut hasher = DefaultHasher::new();
        self.hash(hasher);
        Abstraction::new(hasher.finish())
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

impl Hash for Histogram {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in self.counts.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl PartialEq for Histogram {
    fn eq(&self, other: &Self) -> bool {
        self.counts == other.counts
    }
}

impl Eq for Histogram {
    //
}
