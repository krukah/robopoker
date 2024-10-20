use super::density::Density;
use super::measure::Measure;
use super::support::Support;

pub trait Coupling {
    type X: Support;
    type Y: Support;
    type M: Measure<X = Self::X, Y = Self::Y>;
    type P: Density<X = Self::X>;
    type Q: Density<X = Self::Y>;
    fn flow(&self, x: &Self::X, y: &Self::Y) -> f32;
    /// default ::cost() implemenation assumes that we have flow(x, y
    /// available cheaply enough that we can doubly-integrate
    /// over the support of joint distribution.
    ///
    /// in practice, our optimal cost implmentations (both Metric and
    /// Equity) calculate flow(x, y) lazily and in a way that doesn't
    /// make sense to integrate over the support of the joint distribution.
    ///
    /// Equity uses simple O(N) integration of total variation
    /// Metric uses greedy approximation of EMD.
    fn cost(&self, p: &Self::P, q: &Self::Q, m: &Self::M) -> f32 {
        let mut cost = 0.;
        for x in p.support() {
            for y in q.support() {
                let dx = p.density(x);
                let dy = q.density(y);
                let area = m.distance(x, y);
                let flux = self.flow(x, y);
                cost += area * flux * dx * dy;
            }
        }
        cost
    }
}
