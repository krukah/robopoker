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
