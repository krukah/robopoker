use super::abstraction::Abstraction;
use super::kmeans::KMeans;
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// distribution over arbitrary type T
pub struct Histogram<T> {
    n: usize,
    counts: HashMap<T, usize>,
}

impl<T> Histogram<T>
where
    T: Hash + Eq + Copy,
{
    fn weight(&self, x: &T) -> f32 {
        *self.counts.get(x).unwrap_or(&0) as f32 / self.n as f32
    }
    fn domain(&self) -> Vec<&T> {
        self.counts.keys().collect()
    }

    /// earth mover's distance
    pub fn emd<D>(this: &Self, that: &Self, distance: D) -> f32
    where
        D: Fn(&'_ T, &'_ T) -> f32,
    {
        let n = this.counts.len();
        let m = that.counts.len();
        let mut cost = 0.0;
        let mut extra = HashMap::<T, f32>::new();
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
                let d = distance(this_key, that_key);
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
    /// produce unique signatures for each histogram
    pub fn signature(&self) -> Abstraction {
        let ref mut hasher = DefaultHasher::new();
        self.hash(hasher);
        Abstraction::new(hasher.finish())
    }
}

impl<T> KMeans for Histogram<T>
where
    T: Hash + Eq + Copy,
{
    fn clusters(points: Vec<&Self>) -> Vec<Self> {
        // reset centroids
        let mut centroids: Vec<Self> = Self::initials();
        let k = centroids.len();
        for _ in 0..1_000 {
            // reset clusters
            let mut clusters: Vec<Vec<&Self>> = vec![vec![]; k];
            for x in points.iter() {
                // scan across all observations
                let mut position = 0usize;
                let mut minimium = i32::MAX;
                for (i, y) in centroids.iter().enumerate() {
                    // assign to nearest cenroid
                    let distance = Self::distance(x, y);
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
                .map(|points| Self::centroid(points))
                .collect::<Vec<Self>>();
        }
        centroids
    }
    fn centroid(histograms: Vec<&Self>) -> Self {
        let mut sum = Self::from(vec![]);
        for histogram in histograms {
            for (key, value) in histogram.counts.iter() {
                let mut count = *sum.counts.entry(*key).or_insert(0);
                count += value;
                sum.n += value;
            }
        }
        sum
    }
    fn distance(_: &Self, _: &Self) -> i32 {
        todo!("implement custom earth mover's distance introduced by the paper")
    }
    fn initials() -> Vec<Self> {
        todo!("implement k-means++ initialization")
    }
}

impl<T> From<Vec<T>> for Histogram<T>
where
    T: Hash + Eq,
{
    fn from(abstractions: Vec<T>) -> Self {
        let n = abstractions.len();
        let mut counts = HashMap::new();
        for abs in abstractions {
            *counts.entry(abs).or_insert(0usize) += 1;
        }
        Self { n, counts }
    }
}

impl<T> Hash for Histogram<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for x in self.counts.keys() {
            x.hash(state);
            todo!("use a btree to maintain order");
        }
    }
}

impl<T> PartialEq for Histogram<T>
where
    T: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n;
        todo!("be fr, compare the histograms")
    }
}

impl<T> Eq for Histogram<T>
where
    T: Eq,
{
    //
}
