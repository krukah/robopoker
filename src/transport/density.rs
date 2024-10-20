use super::support::Support;

pub trait Density {
    type X: Support;
    fn density(&self, x: &Self::X) -> f32;
    fn support(&self) -> impl Iterator<Item = &Self::X>;
}
