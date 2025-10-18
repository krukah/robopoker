/// This trait defines the behavior necessary for a Point in a k-means
/// clustering algorithm. This idea of "absorbing" other points is a generalization'
/// of taking a mean, but it allows for the "addition" operation to be decoupled from
/// the "division" operation that we think of when taking a mean of N samples.
pub trait Absorb: Default {
    fn absorb(self, other: &Self) -> Self;
    fn engulf(&mut self, other: &Self);
}

impl Absorb for crate::clustering::Histogram {
    fn absorb(self, other: &Self) -> Self {
        self.absorb(other)
    }
    fn engulf(&mut self, other: &Self) {
        self.engulf(other);
    }
}
