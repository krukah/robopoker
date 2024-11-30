use super::centroid::Centroid;
use crate::cards::isomorphism::Isomorphism;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;

/// intermediate data structure to reference during kmeans
/// as we compute the Wasserstein distance between
/// `Equivalence`s and the available `Abstraction`s > `Centroid`s > `Histogram`s
#[derive(Default)]
pub struct IsomorphismSpace(BTreeMap<Isomorphism, Histogram>);

impl From<BTreeMap<Isomorphism, Histogram>> for IsomorphismSpace {
    fn from(map: BTreeMap<Isomorphism, Histogram>) -> Self {
        Self(map)
    }
}

impl IsomorphismSpace {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn values(&self) -> impl Iterator<Item = &Histogram> {
        self.0.values()
    }
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (&Isomorphism, &Histogram)> {
        self.0.par_iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Isomorphism, &mut Histogram)> {
        self.0.iter_mut()
    }
}

/// intermediate data structure to mutate during kmeans
/// as `Equivalence`s become assigned to `Abstraction`s.
#[derive(Default)]
pub struct AbstractionSpace(BTreeMap<Abstraction, Centroid>);

impl AbstractionSpace {
    /// during initialization, add a distance-weighted
    /// Histogram to the kmeans clusters
    pub fn expand(&mut self, abs: Abstraction, histogram: Histogram) {
        self.0.insert(abs, Centroid::from(histogram));
    }

    /// absorb a `Histogram` into an `Abstraction`
    pub fn absorb(&mut self, abstraction: &Abstraction, histogram: &Histogram) {
        self.0
            .get_mut(abstraction)
            .expect("abstraction generated during initialization")
            .absorb(histogram);
    }

    pub fn orphans(&self) -> Vec<Abstraction> {
        self.0
            .iter()
            .filter(|(_, c)| c.is_empty())
            .map(|(a, _)| a)
            .cloned()
            .collect::<Vec<Abstraction>>()
    }

    pub fn clear(&mut self) {
        for (_, centroid) in self.0.iter_mut() {
            centroid.reset();
        }
    }

    // shallow accessors

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, a: &Abstraction) -> Option<&Centroid> {
        self.0.get(a)
    }
    pub fn keys(&self) -> impl Iterator<Item = &Abstraction> {
        self.0.keys()
    }
    pub fn par_iter(&self) -> impl ParallelIterator<Item = (&Abstraction, &Centroid)> {
        self.0.par_iter()
    }
}
