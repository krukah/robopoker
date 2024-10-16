use super::centroid::Centroid;
use crate::cards::isomorphism::Isomorphism;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use std::collections::BTreeMap;

/// intermediate data structure to reference during kmeans
/// as we compute the Wasserstein distance between
/// `Equivalence`s and the available `Abstraction`s > `Centroid`s > `Histogram`s
#[derive(Default)]
pub struct ObservationSpace(pub BTreeMap<Isomorphism, Histogram>);

/// intermediate data structure to mutate during kmeans
/// as `Equivalence`s become assigned to `Abstraction`s.
#[derive(Default)]
pub struct AbstractionSpace(pub BTreeMap<Abstraction, Centroid>);

impl AbstractionSpace {
    /// during initialization, add a distance-weighted
    /// Histogram to the kmeans clusters
    pub fn expand(&mut self, histogram: Histogram) {
        self.0
            .insert(Abstraction::random(), Centroid::from(histogram));
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
}
