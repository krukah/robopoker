use super::density::Density;
use super::measure::Measure;
use super::support::Support;

pub trait Coupling {
    type X: Support;
    type Y: Support;
    type M: Measure<X = Self::X, Y = Self::Y>;
    type P: Density<S = Self::X>;
    type Q: Density<S = Self::Y>;

    /// default ::cost() implemenation assumes that we have flow(x, y
    /// available cheaply enough that we can doubly-integrate
    /// over the support of joint distribution.
    ///
    /// in practice, our optimal cost implmentations (both Metric and
    /// Equity) calculate flow(x, y) lazily and in a way that doesn't
    /// make sense to integrate over the support of the joint distribution.
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32;

    ///
    /// Equity uses simple O(N) integration of total variation
    /// Metric uses greedy approximation of EMD.
    fn cost(&self) -> f32;
}
