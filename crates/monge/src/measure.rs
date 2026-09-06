use super::support::Support;

/// Ground metric between elements of two support spaces — the per-unit cost
/// that optimal transport minimizes over. `X` and `Y` are usually the same
/// type, but stay distinct to allow heterogeneous transport problems.
pub trait Measure {
    /// Source support space.
    type X: Support;
    /// Target support space.
    type Y: Support;
    /// Returns the cost of transporting mass from `x` to `y`.
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32;
}
