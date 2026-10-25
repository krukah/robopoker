use crate::*;
use mccfr::*;
use pokerkit::*;
use rand::Rng;
use rand::SeedableRng;
use std::collections::BTreeMap;

/// Fixed-capacity uniform sample over an unbounded stream (Vitter's Algorithm R): Deep CFR's
/// memories `M_V,p` and `M_Π`, which keep every iteration's rows equally likely to survive.
#[derive(Debug, Clone)]
pub struct Reservoir<X> {
    items: Vec<X>,
    capacity: usize,
    seen: usize,
    rng: rand::rngs::SmallRng,
}

impl<X> Reservoir<X> {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            capacity,
            seen: 0,
            rng: rand::rngs::SmallRng::seed_from_u64(capacity as u64),
        }
    }

    /// The `n`-th arrival evicts a uniform resident with probability `capacity / n`.
    pub fn insert(&mut self, item: X) {
        if self.len() < self.capacity() {
            self.items.push(item);
        } else {
            let seen = self.seen();
            let index = self.rng().random_range(0..=seen);
            self.items.get_mut(index).map(|x| *x = item);
        }
        self.tick();
    }

    pub fn iter(&self) -> impl Iterator<Item = &X> {
        self.items().iter()
    }

    pub fn len(&self) -> usize {
        self.items().len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn seen(&self) -> usize {
        self.seen
    }

    fn items(&self) -> &[X] {
        &self.items
    }

    fn tick(&mut self) {
        self.seen += 1;
    }

    fn rng(&mut self) -> &mut rand::rngs::SmallRng {
        &mut self.rng
    }
}

impl<F, K> Reservoir<Sample<F, K>>
where
    F: CfrInfo,
    K: Meaning,
{
    /// `argmin_f Σ t · Σ_a (y(I,a) − f(I,a))²` over tables `f`: per edge, `Σ t·y / Σ t`.
    pub fn regress(&self) -> FxHashMap<F, Over<EdgeOf<F>, K>> {
        self.iter()
            .fold(FxHashMap::<F, (Utility, BTreeMap<EdgeOf<F>, Utility>)>::default(), |mut acc, sample| {
                let (mass, sums) = acc.entry(*sample.info()).or_default();
                *mass += sample.weight();
                sample
                    .target()
                    .iter()
                    .for_each(|&(e, y)| *sums.entry(e).or_default() += sample.weight() * y);
                acc
            })
            .into_iter()
            .map(|(info, (mass, sums))| (info, sums.into_iter().map(|(e, s)| (e, s / mass)).collect()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_bounds_residents() {
        let mut memory = Reservoir::new(1 << 6);
        (0..1 << 12).for_each(|x| memory.insert(x));
        assert_eq!(memory.len(), 1 << 6);
        assert_eq!(memory.seen(), 1 << 12);
    }

    #[test]
    fn residents_are_uniform_over_arrivals() {
        let mut memory = Reservoir::new(1 << 10);
        (0..1 << 16).for_each(|x| memory.insert(x));
        let late = memory.iter().filter(|&&x| x >= 1 << 15).count() as f32 / memory.len() as f32;
        assert!((late - 0.5).abs() < 0.1, "late arrivals hold {late:.3} of the reservoir, not ~0.5");
    }
}
