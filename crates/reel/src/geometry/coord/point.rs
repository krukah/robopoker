/// A point in screen space. Only [`crate::Orient`] manufactures
/// these from abstract (layer, slot) coordinates; everything upstream of
/// the projection reasons in axis-free terms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    /// This point translated by (dx, dy).
    pub fn shift(self, dx: f64, dy: f64) -> Self {
        Self::new(self.x() + dx, self.y() + dy)
    }
}
