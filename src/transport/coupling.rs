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
