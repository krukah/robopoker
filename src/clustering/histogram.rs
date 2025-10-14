use crate::cards::observation::Observation;
use crate::gameplay::abstraction::Abstraction;
use crate::transport::density::Density;
use crate::Arbitrary;
use crate::Equity;
use crate::Probability;
use std::collections::BTreeMap;
use std::ops::AddAssign;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Histogram {
    mass: usize,
    counts: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn set(&mut self, abs: Abstraction, count: usize) {
        self.counts.insert(abs, count);
        self.mass += count;
    }
    /// the weight of a given Abstraction. returns 0 if the Abstraction was never witnessed.
    pub fn density(&self, x: &Abstraction) -> Probability {
        self.counts.get(x).copied().unwrap_or(0usize) as f32 / self.mass as f32
    }
    /// all witnessed Abstractions in the support
    pub fn support(&self) -> impl Iterator<Item = &Abstraction> {
        self.counts.keys()
    }
    /// size of the support
    pub fn n(&self) -> usize {
        self.counts.len()
    }

    /// insert the Abstraction into our support,
    /// incrementing its local weight,
    /// incrementing our global norm.
    pub fn increment(mut self, abstraction: Abstraction) -> Self {
        self.mass.add_assign(1usize);
        self.counts
            .entry(abstraction)
            .or_insert(0usize)
            .add_assign(1usize);
        self
    }
    /// absorb the other histogram into this one.
    pub fn absorb(mut self, other: &Self) -> Self {
        self.engulf(other);
        self
    }

    pub fn engulf(&mut self, other: &Self) {
        self.mass += other.mass;
        for (key, count) in other.counts.iter() {
            self.counts.entry(*key).or_insert(0usize).add_assign(*count);
        }
    }

    /// it is useful in EMD calculation
    /// to know if we're dealing with ::Equity or ::Random
    /// Abstraction variants, so we expose this method to
    /// infer the type of Abstraction contained by this Histogram.
    pub fn peek(&self) -> &Abstraction {
        self.counts.keys().next().expect("non empty histogram")
    }
    /// exhaustive calculation of all
    /// possible Rivers and Showdowns,
    /// naive to strategy of course.
    pub fn equity(&self) -> Equity {
        assert!(matches!(self.peek(), Abstraction::Percent(_)));
        self.pdf().iter().map(|(x, y)| x * y).sum()
    }
    /// this yields the posterior equity distribution
    /// at Street::Turn.
    /// this is the only street we explicitly can calculate
    /// the Probability of transitioning into a Probability
    ///     Probability -> Probability
    /// vs  Probability -> Abstraction
    /// hence a distribution over showdown equities.
    pub fn pdf(&self) -> Vec<(Equity, Probability)> {
        assert!(matches!(self.peek(), Abstraction::Percent(_)));
        self.counts
            .iter()
            .map(|(&key, &value)| (key, value as f32 / self.mass as f32))
            .map(|(k, v)| (Equity::from(k), Probability::from(v)))
            .collect()
    }

    /// owned vector of Abstractions and their densities
    /// sorted by density in descending order (most likely first)
    pub fn distribution(&self) -> Vec<(Abstraction, Probability)> {
        let mut distribution = self
            .support()
            .copied()
            .map(|abs| (abs, self.density(&abs)))
            .collect::<Vec<_>>();
        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        distribution
    }
}

impl From<Observation> for Histogram {
    fn from(ref turn: Observation) -> Self {
        assert!(turn.street() == crate::cards::street::Street::Turn);
        turn.children()
            .map(|river| river.equity())
            .map(Abstraction::from)
            .fold(Self::default(), |hist, abs| hist.increment(abs))
    }
}

impl From<Vec<Abstraction>> for Histogram {
    fn from(a: Vec<Abstraction>) -> Self {
        a.into_iter()
            .fold(Self::default(), |hist, abs| hist.increment(abs))
    }
}

impl Density for Histogram {
    type Support = Abstraction;
    fn density(&self, x: &Self::Support) -> f32 {
        self.density(x)
    }
    fn support(&self) -> impl Iterator<Item = &Self::Support> {
        self.support()
    }
}

impl Arbitrary for Histogram {
    fn random() -> Self {
        const S: usize = 16;
        const N: usize = 64;
        (0..)
            .map(|_| Abstraction::random())
            .filter(|a| a.street() == crate::cards::street::Street::Flop)
            .take(S)
            .collect::<Vec<_>>()
            .into_iter()
            .cycle()
            .filter(|_| rand::random::<bool>())
            .take(N)
            .fold(Self::default(), |h, a| h.increment(a))
    }
}

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(matches!(self.peek(), Abstraction::Percent(_)));
        // 1. interpret each key of the Histogram as probability
        // 2. they should already be sorted bc BTreeMap
        let ref pdf = self.pdf();
        // 3. Create 32 bins for the x-axis
        let n_x_bins = 32;
        let ref mut bins = vec![0.0; n_x_bins];
        for (key, value) in pdf {
            let x = key * n_x_bins as f32;
            let x = x.floor() as usize;
            let x = x.min(n_x_bins - 1);
            bins[x] += value;
        }
        // 4. Print the histogram
        writeln!(f)?;
        let n_y_bins = 10;
        for y in (1..=n_y_bins).rev() {
            for bin in bins.iter().copied() {
                if bin >= y as f32 / n_y_bins as f32 {
                    write!(f, "█")?;
                } else if bin >= y as f32 / n_y_bins as f32 - 0.75 / n_y_bins as f32 {
                    write!(f, "▆")?;
                } else if bin >= y as f32 / n_y_bins as f32 - 0.50 / n_y_bins as f32 {
                    write!(f, "▄")?;
                } else if bin >= y as f32 / n_y_bins as f32 - 0.25 / n_y_bins as f32 {
                    write!(f, "▂")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        // 5. Print x-axis
        for _ in 0..n_x_bins {
            write!(f, "-")?;
        }
        // 6. flush to STDOUT
        Ok(())
    }
}
