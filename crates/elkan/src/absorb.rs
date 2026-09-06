/// K-means centroid computation via incremental aggregation, generalizing the
/// mean: combine samples without tracking a count (for histograms, merge
/// probability mass instead of averaging coordinates).
///
/// `absorb` must be associative and commutative, and absorbing N points into
/// `identity` must yield their centroid in whatever sense fits the point type.
pub trait Absorb {
    /// Returns the identity element (zero histogram, etc.).
    fn identity(&self) -> Self;
    /// Combines this point with another, producing a merged result.
    fn absorb(self, other: &Self) -> Self;
}
