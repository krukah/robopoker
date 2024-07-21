#![allow(dead_code)]

use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::observation::Observation;
use crate::cards::board::Street;
use std::collections::HashMap;
use std::vec;

struct Layer {
    street: Street,
    metric: HashMap<Pair, f32>,
    kmeans: HashMap<Observation, Abstraction>,
}

impl Layer {
    /// The River layer is at the bottom of the hierarchy.
    pub fn river() -> Self {
        let street = Street::Rive;
        let kmeans = Observation::predecessors(Street::Show)
            .into_iter()
            .map(|obs| (obs, Abstraction::from(obs.equity())))
            .collect::<HashMap<_, _>>();
        let bins = (0..Abstraction::BUCKETS)
            .map(|i| Abstraction::from(i))
            .collect::<Vec<_>>();
        let mut metric = HashMap::new();
        for (i, a) in bins.iter().enumerate() {
            for (j, b) in bins.iter().enumerate() {
                if i > j {
                    let key = Pair::from((*a, *b));
                    let distance = (i - j) as f32;
                    metric.insert(key, distance);
                }
            }
        }
        Self {
            street,
            metric,
            kmeans,
        }
    }

    /// Every other layer is generated from its next lowest level of abstraction.
    pub fn upper(lower: &Self) -> Self {
        let histograms = lower.histograms();
        let ref centroids = lower.k_means(histograms.values().collect(), 100);
        let kmeans = lower.upper_kmeans(centroids, histograms);
        let metric = lower.upper_metric(centroids);
        let street = lower.upper_street();
        Self {
            street,
            metric,
            kmeans,
        }
    }

    fn histograms(&self) -> HashMap<Observation, Histogram> {
        Observation::predecessors(self.street)
            .into_iter()
            .map(|o| (o, self.histogram(&o)))
            .collect::<HashMap<_, _>>()
    }
    fn histogram(&self, observation: &Observation) -> Histogram {
        Histogram::from(
            observation
                .successors()
                .into_iter()
                .map(|ref o| self.abstraction(o))
                .collect::<Vec<_>>(),
        )
    }
    fn abstraction(&self, observation: &Observation) -> Abstraction {
        self.kmeans
            .get(observation)
            .copied()
            .expect("we should have computed signatures previously")
    }
    fn distance(&self, a: &Abstraction, b: &Abstraction) -> f32 {
        let ref index = Pair::from((*a, *b));
        self.metric
            .get(index)
            .copied()
            .expect("we should have computed distances previously")
    }
    fn emd(&self, this: &Histogram, that: &Histogram) -> f32 {
        let n = this.size();
        let m = that.size();
        let mut cost = 0.0;
        let mut extra = HashMap::new();
        let mut goals = vec![1.0 / n as f32; n];
        let mut empty = vec![false; n];
        for i in 0..m {
            for j in 0..n {
                if empty[j] {
                    continue;
                }
                let this_key = this.domain()[j];
                let that_key = that.domain()[i];
                let spill = extra
                    .get(that_key)
                    .cloned()
                    .or_else(|| Some(that.weight(that_key)))
                    .expect("key is somewhere");
                if spill == 0f32 {
                    continue;
                }
                let d = self.distance(this_key, that_key);
                let bonus = spill - goals[j];
                if (bonus) < 0f32 {
                    extra.insert(*that_key, 0f32);
                    cost += d * bonus as f32;
                    goals[j] -= bonus as f32;
                } else {
                    extra.insert(*that_key, bonus);
                    cost += d * goals[j];
                    goals[j] = 0.0;
                    empty[j] = true;
                }
            }
        }
        cost
    }

    // builder methods for the next layer
    fn upper_street(&self) -> Street {
        match self.street {
            Street::Pref => panic!("no previous street"),
            Street::Flop => Street::Pref,
            Street::Turn => Street::Flop,
            Street::Rive => Street::Turn,
            Street::Show => panic!("this variant might be undesirable"),
        }
    }
    fn upper_kmeans(
        &self,
        centroids: &Vec<Histogram>,
        histograms: HashMap<Observation, Histogram>,
    ) -> HashMap<Observation, Abstraction> {
        let mut abstractions = HashMap::new();
        for (observation, ref histogram) in histograms {
            let mut minimium = f32::MAX;
            let mut neighbor = histogram;
            for ref centroid in centroids {
                let distance = self.emd(histogram, centroid);
                if distance < minimium {
                    minimium = distance;
                    neighbor = centroid;
                }
            }
            abstractions.insert(observation, Abstraction::from(neighbor));
        }
        abstractions
    }
    fn upper_metric(&self, centroids: &Vec<Histogram>) -> HashMap<Pair, f32> {
        let mut distances = HashMap::new();
        for (i, a) in centroids.iter().enumerate() {
            for (j, b) in centroids.iter().enumerate() {
                if i > j {
                    let key = Pair::from((Abstraction::from(a), Abstraction::from(b)));
                    let distance = self.emd(a, b);
                    distances.insert(key, distance);
                }
            }
        }
        distances
    }

    // k-means clustering
    fn k_means(&self, histograms: Vec<&Histogram>, t: usize) -> Vec<Histogram> {
        let mut centroids = self.guesses();
        let k = centroids.len();
        for _ in 0..t {
            let mut clusters: Vec<Vec<&Histogram>> = vec![vec![]; k];
            for x in histograms.iter() {
                let mut position = 0usize;
                let mut minimium = f32::MAX;
                for (i, y) in centroids.iter().enumerate() {
                    let distance = self.emd(x, y);
                    if distance < minimium {
                        minimium = distance;
                        position = i;
                    }
                }
                clusters
                    .get_mut(position)
                    .expect("position in range")
                    .push(x);
            }
            centroids = clusters
                .into_iter()
                .map(|points| Histogram::centroid(points))
                .collect::<Vec<Histogram>>();
        }
        centroids
    }
    fn guesses(&self) -> Vec<Histogram> {
        let _k = match self.street {
            Street::Pref => 2,
            Street::Flop => 4,
            Street::Turn => 8,
            Street::Rive => 16,
            Street::Show => panic!(),
        };
        todo!("implement k-means++ initialization")
    }
}

/// A unique identifier for a pair of abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct Pair(u64);
impl From<(Abstraction, Abstraction)> for Pair {
    fn from((a, b): (Abstraction, Abstraction)) -> Self {
        Self(u64::from(a) ^ u64::from(b))
    }
}
