pub trait Absorb: Default {
    fn absorb(self, other: &Self) -> Self;
}
impl Absorb for crate::clustering::Histogram {
    fn absorb(self, other: &Self) -> Self {
        self.puma(other)
    }
}
