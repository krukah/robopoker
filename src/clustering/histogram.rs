use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::transport::density::Density;
use crate::Equity;
use crate::Probability;
use std::collections::BTreeMap;
use std::ops::AddAssign;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
#[derive(Debug, Default, Clone)]
pub struct Histogram {
    mass: usize,
    contribution: BTreeMap<Abstraction, usize>,
}

impl Density for Histogram {
    type X = Abstraction;
    fn density(&self, x: &Self::X) -> f32 {
        self.weight(x)
    }
    fn support(&self) -> impl Iterator<Item = &Self::X> {
        self.support()
    }
}

impl Histogram {
    /// the weight of a given Abstraction.
    /// returns 0 if the Abstraction was never witnessed.
    pub fn weight(&self, abstraction: &Abstraction) -> f32 {
        self.contribution
            .get(abstraction)
            .copied()
            .unwrap_or(0usize) as f32
            / self.mass as f32
    }

    /// all witnessed Abstractions.
    /// treat this like an unordered array
    /// even though we use BTreeMap for struct.
    pub fn support(&self) -> impl Iterator<Item = &Abstraction> {
        self.contribution.keys()
    }
    pub fn normalized(&self) -> BTreeMap<Abstraction, f32> {
        self.contribution
            .iter()
            .map(|(&a, &count)| (a, count as f32 / self.mass as f32))
            .collect()
    }

    /// useful only for k-means edge case of centroid drift
    pub fn is_empty(&self) -> bool {
        assert!(self.contribution.is_empty() == (self.mass == 0));
        self.contribution.is_empty()
    }

    /// size of the support
    pub fn size(&self) -> usize {
        self.contribution.len()
    }

    /// insert the Abstraction into our support,
    /// incrementing its local weight,
    /// incrementing our global norm.
    pub fn witness(mut self, abstraction: Abstraction) -> Self {
        self.mass.add_assign(1usize);
        self.contribution
            .entry(abstraction)
            .or_insert(0usize)
            .add_assign(1usize);
        self
    }

    /// empty the whole thing,
    /// reset the norm to zero,
    /// clear the weights
    pub fn destroy(&mut self) {
        self.mass = 0;
        self.contribution.clear();
    }

    /// absorb the other histogram into this one.
    ///
    /// TODO:
    /// Note that this implicitly assumes sum normalizations are the same,
    /// which should hold for now...
    /// until we implement Observation isomorphisms!
    pub fn absorb(&mut self, other: &Self) {
        self.mass += other.mass;
        for (key, count) in other.contribution.iter() {
            self.contribution
                .entry(key.to_owned())
                .or_insert(0usize)
                .add_assign(count.to_owned());
        }
    }

    /// it is useful in EMD calculation
    /// to know if we're dealing with ::Equity or ::Random
    /// Abstraction variants, so we expose this method to
    /// infer the type of Abstraction contained by this Histogram.
    pub fn peek(&self) -> &Abstraction {
        self.contribution
            .keys()
            .next()
            .expect("non empty histogram")
    }

    /// exhaustive calculation of all
    /// possible Rivers and Showdowns,
    /// naive to strategy of course.
    pub fn equity(&self) -> Equity {
        assert!(matches!(self.peek(), Abstraction::Equity(_)));
        self.distribution().iter().map(|(x, y)| x * y).sum()
    }

    /// this yields the posterior equity distribution
    /// at Street::Turn.
    /// this is the only street we explicitly can calculate
    /// the Probability of transitioning into a Probability
    ///     Probability -> Probability
    /// vs  Probability -> Abstraction
    /// hence a distribution over showdown equities.
    pub fn distribution(&self) -> Vec<(Equity, Probability)> {
        assert!(matches!(self.peek(), Abstraction::Equity(_)));
        self.contribution
            .iter()
            .map(|(&key, &value)| (key, value as f32 / self.mass as f32))
            .map(|(k, v)| (Equity::from(k), Probability::from(v)))
            .collect()
    }
}

impl From<Observation> for Histogram {
    fn from(ref turn: Observation) -> Self {
        assert!(turn.street() == crate::cards::street::Street::Turn);
        turn.children()
            .map(|river| Abstraction::from(river.equity()))
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
}

impl From<Vec<Abstraction>> for Histogram {
    fn from(a: Vec<Abstraction>) -> Self {
        a.into_iter()
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
}

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(matches!(self.peek(), Abstraction::Equity(_)));
        // 1. interpret each key of the Histogram as probability
        // 2. they should already be sorted bc BTreeMap
        let ref distribution = self.distribution();
        // 3. Create 32 bins for the x-axis
        let n_x_bins = 32;
        let ref mut bins = vec![0.0; n_x_bins];
        for (key, value) in distribution {
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
