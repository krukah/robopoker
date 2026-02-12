use super::support::Support;

/// Ground metric between elements of two support spaces.
///
/// Defines the cost of transporting one unit of mass from a point in
/// the source space to a point in the target space. This is the "ground cost"
/// that optimal transport algorithms minimize over.
///
/// # Type Parameters
///
/// - `X` — Source support space (e.g., source distribution's abstraction buckets)
/// - `Y` — Target support space (e.g., target distribution's abstraction buckets)
///
/// While `X` and `Y` are often the same type, the trait supports heterogeneous
/// transport problems where source and target live in different spaces.
///
/// # Implementations
///
/// - **Equity**: Absolute difference `|x - y|` between equity values
/// - **Metric**: Precomputed pairwise distances from clustering
pub trait Measure {
    /// Source support space.
    type X: Support;
    /// Target support space.
    type Y: Support;
    /// Returns the cost of transporting mass from `x` to `y`.
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32;
}
