use std::hash::Hash;

/// k-means clustering requires
/// - initial guess
/// - distance metric
/// - centroid calculation
/// - k
pub trait KMeans: Sized + Eq + Hash {
    fn clusters(refs: Vec<&Self>) -> Vec<Self>;
    fn distance(this: &Self, that: &Self) -> i32;
    fn initials() -> Vec<Self>;
    fn centroid(cluster: Vec<&Self>) -> Self;
}
