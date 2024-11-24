use super::coupling::Coupling;
use crate::Probability;
use crate::Utility;

pub trait Sinkhorn: Coupling {
    // hyperparameters
    fn e(&self) -> f32;
    fn i(&self) -> usize;

    // accessors to:
    // - p, the source distribution
    // - q, the target distribution
    // - m, the metric, which is used to calculate:
    // - k, the kernel, which is the exponential of the negative distance
    fn p(&self) -> &Self::P;
    fn q(&self) -> &Self::Q;
    fn m(&self) -> &Self::M;
    fn k(&self) -> &Self::M;

    /// current LHS potential
    fn prev_u(&self) -> &Self::P;
    /// current RHS potential
    fn prev_v(&self) -> &Self::Q;

    // i.e. density(x) / marginal(x)
    fn scale_x(&self, x: &Self::X) -> Probability;
    // i.e. density(y) / marginal(y)
    fn scale_y(&self, y: &Self::Y) -> Probability;

    /// cumulative flow out of LHS element under current potentials
    fn marginal_x(&self, x: &Self::X) -> Probability;
    /// cumulative flow out of RHS element under current potentials
    fn marginal_y(&self, y: &Self::Y) -> Probability;

    /// next LHS potential
    fn next_u(&self) -> Self::P;
    /// next RHS potential
    fn next_v(&self) -> Self::Q;

    /// the final optimal transport plan. type is equivalent to Measure, given that we map (X, Y) ↦ ℝ
    fn last_k(&self) -> Self::M;
    /// the cost of the final transport plan
    fn cost_k(&self) -> Utility;
}
