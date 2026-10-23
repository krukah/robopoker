/// An axis-aligned box in screen space, produced by projecting an abstract
/// (layer, slot) extent through an [`crate::Orient`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn w(&self) -> f64 {
        self.w
    }

    pub fn h(&self) -> f64 {
        self.h
    }

    /// This rect translated by (dx, dy).
    pub fn shift(self, dx: f64, dy: f64) -> Self {
        Self::new(self.x() + dx, self.y() + dy, self.w(), self.h())
    }
}
