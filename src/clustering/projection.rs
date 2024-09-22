use crate::cards::observation::NodeObservation;
use crate::clustering::abstraction::NodeAbstraction;
use crate::clustering::histogram::Histogram;
use std::collections::BTreeMap;

/// Enables inter- and intra-layer projections for hierarchical clustering.
///
/// Defines methods for translating the outer(inner) observations into abstractions(distributions) in the outer layer
/// It is crucial for maintaining the hierarchical structure of the clustering algorithm
/// and [normalizing, compressing, generalizing] potential-awareness between different streets or levels of abstraction.
/// All expectations are such that Observation::all(street) and obs.outnodes() will project perfectly across layers
pub trait Projection {
    fn project(&self, inner: NodeObservation) -> Histogram; // (_, BTreeMap<Abstraction, usize>)
    fn convert(&self, outer: NodeObservation) -> NodeAbstraction;
}

impl Projection for BTreeMap<NodeObservation, (Histogram, NodeAbstraction)> {
    fn project(&self, ref inner: NodeObservation) -> Histogram {
        inner
            .outnodes()
            .into_iter()
            .map(|outer| self.convert(outer))
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
    fn convert(&self, ref outer: NodeObservation) -> NodeAbstraction {
        self.get(outer)
            .expect("abstraction calculated in previous layer")
            .1
            .clone()
    }
}
