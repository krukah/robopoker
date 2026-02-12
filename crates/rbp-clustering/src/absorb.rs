/// Trait for k-means centroid computation via incremental aggregation.
///
/// The "absorb" pattern generalizes mean computation: we can incrementally
/// combine samples without tracking the count explicitly. For histograms,
/// this means merging probability mass rather than averaging coordinates.
///
/// # Required Methods
///
/// - `identity()` — Returns the neutral element for absorption
/// - `absorb()` — Combines two points into one (associative, commutative)
///
/// # Invariant
///
/// After absorbing N points, the result should be the centroid (mean) of
/// those points in whatever sense is appropriate for the point type.
pub trait Absorb {
    /// Returns the identity element (zero histogram, etc.).
    fn identity(&self) -> Self;
    /// Combines this point with another, producing a merged result.
    fn absorb(self, other: &Self) -> Self;
}

impl Absorb for crate::Histogram {
    fn absorb(self, other: &Self) -> Self {
        self.absorb(other)
    }
    fn identity(&self) -> Self {
        Self::empty(self.street())
    }
}
