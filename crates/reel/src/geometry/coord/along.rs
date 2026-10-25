/// A coordinate on the layer axis. Newtyped so it can never be confused
/// with a slot-axis coordinate — swapping the two is a type error, not a
/// silently transposed frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Along(f64);

impl Along {
    pub fn new(at: f64) -> Self {
        Self(at)
    }

    pub fn raw(&self) -> f64 {
        self.0
    }
}
