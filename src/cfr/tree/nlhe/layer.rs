#![allow(dead_code)]

use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::observation::Observation;
use crate::cards::board::Street;
use std::collections::HashMap;

type Centroids<'a> = &'a Vec<Histogram>;
type Projections = HashMap<Observation, Histogram>;
type Mappings = HashMap<Observation, Abstraction>;
type Measures = HashMap<[Abstraction; 2], f32>;

struct Layer {
    street: Street, // probably try to remove this from the struct - find references and unify
    measure: Measures,
    mapping: Mappings,
}

impl Layer {
    fn predecessors(&self) -> Vec<Observation> {
        todo!(
            "
            generate every possible immediately previous observation at this street
            conditional on street
        "
        )
    }
    fn successors(&self, _: &Observation) -> Vec<&Observation> {
        todo!(
            "
            select a range of entries from self abstraction
            OR
            simulate all continuations of this streets
        "
        );
    }

    pub fn bottom() -> Self {
        todo!(
            "
            * generate all possible river Observations
            * map them into Abstraction (via Abstraction::from(Observation))
            * return Layer with Abstraction mapping and distance measure
        "
        )
    }
    pub fn from(lower: &Self) -> Self {
        let projections = lower.project();
        let histograms = projections.values().collect();
        let ref centroids = lower.k_means(histograms, 100);
        Self {
            mapping: lower.upper_mapping(centroids, projections),
            measure: lower.upper_measure(centroids),
            street: Street::next(&lower.street),
        }
    }

    fn project(&self) -> Projections {
        self.predecessors()
            .into_iter()
            .map(|o| (o, self.histogram(&o)))
            .collect::<HashMap<_, _>>()
    }
    fn histogram(&self, observation: &Observation) -> Histogram {
        Histogram::from(
            self.successors(observation)
                .into_iter()
                .map(|o| self.mapping(o))
                .collect::<Vec<_>>(),
        )
    }
    fn mapping(&self, observation: &Observation) -> Abstraction {
        // this shouldnt ened to reference self.streeet. self.street should emerge from  somewhere
        // observation.street() ?
        // layer.street() ?
        // abstraction.street() ?
        // match self.street {
        //    Street::Showdown => Abstraction::from(self.equity(observation)),
        //    _ => self.mapping.get(observation).copied().expect("we should have computed signatures previously"),
        // }
        self.mapping
            .get(observation)
            .copied()
            .expect("we should have computed signatures previously")
    }
    fn measure(&self, a: &Abstraction, b: &Abstraction) -> f32 {
        // match self.street {
        //    Street::Showdown => (u64::from(*a) - u64::from(*b)).abs()
        //    _ => self.measure.get(&[*a, *b]).or_else(|| self.measure.get(&[*b, *a])).copied().expect("we should have computed distances previously"),
        // }
        self.measure
            .get(&[*a, *b])
            .or_else(|| self.measure.get(&[*b, *a]))
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
                let d = self.measure(this_key, that_key);
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
    fn upper_mapping(&self, centroids: Centroids, projections: Projections) -> Mappings {
        let mut abstractions = HashMap::new();
        for (observation, ref histogram) in projections {
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
    fn upper_measure(&self, centroids: Centroids) -> Measures {
        let mut distances = HashMap::new();
        for (i, a) in centroids.iter().enumerate() {
            for (j, b) in centroids.iter().enumerate() {
                if i > j {
                    let key = [Abstraction::from(a), Abstraction::from(b)];
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
        todo!("implement k-means++ initialization")
    }
}

// equity calc
// [Card; 7] -> Strength
// for every villain hand -> Strength
// + 2 if win
// + 1 if tie
// + 0 if lose
// divide by 2 * len()
