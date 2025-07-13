use super::histogram::Histogram;
use super::metric::Metric;
use crate::Energy;

// TODO MOVE OUT TO OTHER FILE WITH KMEANS CLUSTERING CODE
pub trait Clusterable {
    // Probably have this use generics here if possible, + return a simple f32. E.g. some
    // function <T1, T2> that does ((T1, T2, T2) -> f32).
    fn distance(m: &Metric, h1: &Histogram, h2: &Histogram) -> Energy;
    // Probably remove this entirely and just have an implementation of this over in
    // the resulting file defined in terms of the other inputs + the distance function...?
    fn nearest_neighbor(m: &Metric, clusters: Vec<Histogram>, x: &Histogram) -> (usize, f32);
    // Probably also remove this in favor of just some direct input for some future
    // kmeans_cluster function...
    fn points(&self) -> &Vec<Histogram>;
}
