use super::density::Density;
use super::measure::Measure;
use super::support::Support;

/// A transport plan (coupling) between two probability distributions.
///
/// In optimal transport theory, a coupling is a joint distribution π(x,y) whose
/// marginals match the source distribution P and target distribution Q. The
/// transport cost is the expected ground cost under this joint distribution.
///
/// # Type Parameters
///
/// - `X` — Source support space
/// - `Y` — Target support space
/// - `M` — Ground metric defining transport costs
/// - `P` — Source distribution (marginal over X)
/// - `Q` — Target distribution (marginal over Y)
///
/// # Algorithm Contract
///
/// Implementations must ensure that after [`minimize`](Coupling::minimize) is called,
/// [`cost`](Coupling::cost) returns the optimal transport cost.
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
    /// Optimizes the coupling to minimize total transport cost.
    ///
    /// After this method returns, [`cost`](Coupling::cost) yields the optimal
    /// transport cost (Earth Mover's Distance when using L1 ground metric).
    fn minimize(self) -> Self;
    /// Returns the mass transported from `x` to `y` in the coupling.
    ///
    /// For sparse couplings, this may be computed lazily rather than stored
    /// explicitly for all (x, y) pairs.
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32;
    /// Returns the total transport cost of this coupling.
    ///
    /// This is the integral of `flow(x, y) * distance(x, y)` over all pairs.
    /// Different implementations use different strategies:
    /// - **Equity**: O(N) integration via total variation distance
    /// - **Metric**: Greedy EMD approximation with precomputed distances
    fn cost(&self) -> f32;
}
