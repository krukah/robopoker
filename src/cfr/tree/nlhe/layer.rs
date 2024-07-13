use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::observation::Observation;
use crate::cfr::tree::nlhe::kmeans::KMeans;
use std::collections::HashMap;

struct Layer {
    distances: HashMap<[Abstraction; 2], f32>,
    abstractions: HashMap<Observation, Abstraction>,
}

impl Layer {
    pub fn from(inner: &Self) -> Self {
        let histograms = Self::histograms(inner);
        let ref centroids = Histogram::clusters(histograms.values().collect());
        let ref metric = |x: &Abstraction, y: &Abstraction| inner.distance(x, y);
        let distances = Self::measure(centroids, metric);
        let abstractions = Self::mapping(histograms, centroids, metric);
        Self {
            distances,
            abstractions,
        }
    }

    fn measure<D>(means: &Vec<Histogram<Abstraction>>, metric: &D) -> HashMap<[Abstraction; 2], f32>
    where
        D: Fn(&Abstraction, &Abstraction) -> f32,
    {
        let mut distances = HashMap::new();
        for (i, a) in means.iter().enumerate() {
            for (j, b) in means.iter().enumerate() {
                if i > j {
                    let distance = Histogram::emd(a, b, metric);
                    let key = [a.signature(), b.signature()];
                    distances.insert(key, distance);
                }
            }
        }
        distances
    }

    fn mapping<D>(
        histograms: HashMap<Observation, Histogram<Abstraction>>,
        means: &Vec<Histogram<Abstraction>>,
        metric: &D,
    ) -> HashMap<Observation, Abstraction>
    where
        D: Fn(&Abstraction, &Abstraction) -> f32,
    {
        let mut abstractions = HashMap::new();
        for (observation, ref histogram) in histograms {
            let mut minimium = f32::MAX;
            let mut neighbor = histogram;
            for mean in means.iter() {
                let distance = Histogram::emd(histogram, mean, metric);
                if distance < minimium {
                    minimium = distance;
                    neighbor = mean;
                }
            }
            abstractions.insert(observation, neighbor.signature());
        }
        abstractions
    }

    fn distance(&self, a: &Abstraction, b: &Abstraction) -> f32 {
        self.distances.get(&[*a, *b]).copied().unwrap_or_else(|| {
            self.distances.get(&[*b, *a]).copied().unwrap_or_else(|| {
                unreachable!("we should have computed distances previously + we check both orders")
            })
        })
    }
    fn signature(&self, observation: &Observation) -> Abstraction {
        self.abstractions
            .get(observation)
            .copied()
            .expect("we should have computed signatures previously")
    }
    fn histograms(inner: &Self) -> HashMap<Observation, Histogram<Abstraction>> {
        Self::observations()
            .into_iter()
            .map(|observation| {
                (
                    observation,
                    Self::continuations(&observation)
                        .iter()
                        .map(|o| inner.signature(o))
                        .collect::<Vec<_>>()
                        .into(),
                )
            })
            .collect()
    }
    fn continuations(_: &Observation) -> Vec<Observation> {
        todo!("generate all possible continuations at this street")
    }
    fn observations() -> Vec<Observation> {
        todo!("generate all possible observations at this street")
    }
}
