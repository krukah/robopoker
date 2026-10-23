/// A coordinate on the slot axis — the perpendicular twin of
/// [`crate::Along`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cross(f64);

impl Cross {
    pub fn new(at: f64) -> Self {
        Self(at)
    }

    pub fn raw(&self) -> f64 {
        self.0
    }
}
