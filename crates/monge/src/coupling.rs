use super::density::Density;
use super::measure::Measure;
use super::support::Support;

/// A transport plan between two probability distributions: a joint
/// distribution π(x,y) whose marginals match source `P` and target `Q`, with
/// transport cost the expected ground cost under that joint.
///
/// Implementations must guarantee that once [`Self::minimize`] has run,
/// [`Self::cost`] returns the optimal transport cost.
pub trait Coupling {
    /// Source support space.
    type X: Support;
    /// Target support space.
    type Y: Support;
    /// Ground metric for transport costs.
    type M: Measure<X = Self::X, Y = Self::Y>;
    /// Source probability distribution.
    type P: Density<Support = Self::X>;
    /// Target probability distribution.
    type Q: Density<Support = Self::Y>;
    /// Minimize total transport cost — EMD under an L1 ground metric.
    fn minimize(self) -> Self;
    /// Mass transported from `x` to `y`. Sparse couplings may compute this
    /// lazily rather than store every `(x, y)` pair.
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32;
    /// Total transport cost: `flow(x, y) * distance(x, y)` integrated over
    /// all pairs.
    fn cost(&self) -> f32;
}
