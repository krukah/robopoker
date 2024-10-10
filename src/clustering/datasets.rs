use super::centroid::Centroid;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use std::collections::BTreeMap;

/// intermediate data structure to reference during kmeans
/// as we compute the Wasserstein distance between
/// `Observation`s and the available `Abstraction`s > `Centroid`s > `Histogram`s
#[derive(Default)]
pub struct ObservationSpace(pub BTreeMap<Observation, Histogram>);

/// intermediate data structure to mutate during kmeans
/// as `Observation`s become assigned to `Abstraction`s.
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
}
