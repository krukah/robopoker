pub trait Absorb: Default {
    fn absorb(&mut self, other: &Self);
}
impl Absorb for crate::clustering::Histogram {
    fn absorb(&mut self, other: &Self) {
        self.absorb(other);
    }
}
