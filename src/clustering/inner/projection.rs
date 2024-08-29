use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use std::collections::HashMap;

/// Enables inter- and intra-layer projections for hierarchical clustering.
///
/// Defines methods for translating the outer(inner) observations into abstractions(distributions) in the outer layer
/// It is crucial for maintaining the hierarchical structure of the clustering algorithm
/// and [normalizing, compressing, generalizing] potential-awareness between different streets or levels of abstraction.
/// All expectations are such that Observation::all(street) and obs.outnodes() will project perfectly across layers
pub trait Projection {
    fn convert(&self, outer: Observation) -> Abstraction;
    fn project(&self, inner: Observation) -> Histogram;
}
impl Projection for HashMap<Observation, (Histogram, Abstraction)> {
    fn convert(&self, ref outer: Observation) -> Abstraction {
        self.get(outer)
            .expect("abstraction calculated in previous layer")
            .1
            .clone()
    }
    fn project(&self, ref inner: Observation) -> Histogram {
        inner
            .outnodes()
            .into_iter()
            .map(|outer| self.convert(outer))
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
}
