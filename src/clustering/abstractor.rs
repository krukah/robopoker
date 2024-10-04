use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use std::collections::BTreeMap;

/// this is the output of the clustering module
/// it is a massive table of `Observation` -> `Abstraction`.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
#[derive(Default)]
pub struct Abstractor(pub BTreeMap<Observation, Abstraction>);

impl Abstractor {
    /// at a given `Street`,
    /// 1. decompose the `Observation` into all of its next-street `Observation`s,
    /// 2. map each of them into an `Abstraction`,
    /// 3. collect the results into a `Histogram`.
    pub fn projection(&self, inner: &Observation) -> Histogram {
        match inner.street() {
            Street::Turn => inner.clone().into(),
            _ => inner
                .outnodes()
                .into_iter()
                .map(|ref outer| self.abstraction(outer))
                .collect::<Vec<Abstraction>>()
                .into(),
        }
    }

    /// lookup the pre-computed abstraction for the outer observation
    pub fn abstraction(&self, outer: &Observation) -> Abstraction {
        self.0
            .get(outer)
            .cloned()
            .expect("precomputed abstraction mapping")
    }

    /// simple insertion.
    /// can we optimize out this clone though?
    pub fn assign(&mut self, a: &Abstraction, o: &Observation) {
        self.0.insert(o.to_owned(), a.to_owned());
    }
}
