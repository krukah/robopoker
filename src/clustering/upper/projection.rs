use super::histogram::Histogram;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use std::collections::HashMap;

/// Enables inter- and intra-layer projections for hierarchical clustering.
///
/// Defines methods for translating the lower(upper) observations into abstractions(distributions) in the lower layer
/// It is crucial for maintaining the hierarchical structure of the clustering algorithm
/// and [normalizing, compressing, generalizing] potential-awareness between different streets or levels of abstraction.
/// All expectations are such that Observation::all(street) and obs.outnodes() will project perfectly across layers
pub trait Projection {
    fn convert(&self, lower: Observation) -> Abstraction;
    fn project(&self, upper: Observation) -> Histogram;
}
impl Projection for HashMap<Observation, (Histogram, Abstraction)> {
    fn convert(&self, ref lower: Observation) -> Abstraction {
        self.get(lower)
            .expect("abstraction calculated in previous layer")
            .1
            .clone()
    }
    fn project(&self, ref upper: Observation) -> Histogram {
        upper
            .outnodes()
            .into_iter()
            .map(|lower| self.convert(lower))
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
}
