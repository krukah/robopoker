use crate::cards::observation::CardObservation;
use crate::clustering::abstraction::CardAbstraction;
use crate::clustering::histogram::Histogram;
use std::collections::BTreeMap;

/// Enables inter- and intra-layer projections for hierarchical clustering.
///
/// Defines methods for translating the outer(inner) observations into abstractions(distributions) in the outer layer
/// It is crucial for maintaining the hierarchical structure of the clustering algorithm
/// and [normalizing, compressing, generalizing] potential-awareness between different streets or levels of abstraction.
/// All expectations are such that Observation::all(street) and obs.outnodes() will project perfectly across layers
pub trait Projection {
    fn project(&self, inner: CardObservation) -> Histogram; // (_, BTreeMap<Abstraction, usize>)
    fn convert(&self, outer: CardObservation) -> CardAbstraction;
}

impl Projection for BTreeMap<CardObservation, (Histogram, CardAbstraction)> {
    fn project(&self, ref inner: CardObservation) -> Histogram {
        inner
            .outnodes()
            .into_iter()
            .map(|outer| self.convert(outer))
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
    fn convert(&self, ref outer: CardObservation) -> CardAbstraction {
        self.get(outer)
            .expect("abstraction calculated in previous layer")
            .1
            .clone()
    }
}
