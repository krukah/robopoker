use super::{abstraction::Abstraction, histogram::Histogram, observation::Observation};
use crate::cfr::tree::nlhe::kmeans::KMeans;
use std::collections::HashMap;

struct Layer {
    distances: HashMap<[Abstraction; 2], f32>,
    abstractions: HashMap<Observation, Abstraction>,
}

impl Layer {
    pub fn from(inner: Self) -> Self {
        let histograms = Self::observations()
            .into_iter()
            .map(|observation| {
                let histogram: Histogram<_> = Self::continuations(&observation)
                    .iter()
                    .map(|o| inner.abstractions.get(o).expect("idk").clone())
                    .collect::<Vec<_>>()
                    .into();
                (observation, histogram)
            })
            .collect::<HashMap<Observation, Histogram<Abstraction>>>();
        let cluster = Histogram::clusters(histograms.values().collect());
        let mut distances = HashMap::<[Abstraction; 2], f32>::new();
        // something about triangle ineq to speed up distance calculation???
        for a in cluster.iter() {
            for b in cluster.iter() {
                let key = [a.signature(), b.signature()];
                let distance = Histogram::emd(a, b, |x, y| inner.distance(x, y));
                distances.insert(key, distance);
                todo!("constrain i > j");
            }
        }
        // generate OuterAbstractions for the select clusters
        // compute distances across clusters
        // assign observations to abstractions based on their distance calculation
        todo!("compute distances, assign obs -> abs map")
    }
    fn observations() -> Vec<Observation> {
        todo!("generate all possible observations at this street")
    }
    fn abstraction(&self, observation: &Observation) -> Abstraction {
        self.abstractions
            .get(observation)
            .copied()
            .expect("we should have computed signatures previously")
    }
    fn distance(&self, a: &Abstraction, b: &Abstraction) -> f32 {
        self.distances.get(&[*a, *b]).copied().unwrap_or_else(|| {
            self.distances.get(&[*b, *a]).copied().unwrap_or_else(|| {
                panic!("we should have computed distances previously + we check both orders")
            })
        })
    }
    fn continuations(_: &Observation) -> Vec<Observation> {
        todo!("generate all possible continuations at this street")
    }
}
