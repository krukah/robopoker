use super::support::Support;

pub trait Measure {
    type X: Support;
    type Y: Support;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32;
}
