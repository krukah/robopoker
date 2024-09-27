use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use std::collections::BTreeMap;
use std::ops::AddAssign;

/// A distribution over arbitrary Abstractions.
///
/// The sum of the weights is the total number of samples.
/// The weight of an abstraction is the number of times it was sampled.
#[derive(Debug, Default, Clone)]
pub struct Histogram {
    norm: usize,
    weights: BTreeMap<Abstraction, usize>,
}

impl Histogram {
    pub fn weight(&self, abstraction: &Abstraction) -> f32 {
        self.weights.get(abstraction).copied().unwrap_or(0usize) as f32 / self.norm as f32
    }
    pub fn domain(&self) -> Vec<&Abstraction> {
        self.weights.keys().collect()
    }
    pub fn witness(self, abstraction: Abstraction) -> Self {
        let mut this = self;
        this.norm.add_assign(1usize);
        this.weights
            .entry(abstraction)
            .or_insert(0usize)
            .add_assign(1usize);
        this
    }
    pub fn destroy(&mut self) {
        self.norm = 0;
        self.weights.clear();
    }
    /// Absorb the other histogram into this one.
    /// Note that this implicitly assumes sum normalizations are the same,
    /// which should hold until we implement Observation isomorphisms!
    pub fn absorb(&mut self, other: &Self) {
        assert!(self.norm == other.norm);
        self.norm += other.norm;
        for (key, count) in other.weights.iter() {
            self.weights
                .entry(key.to_owned())
                .or_insert(0usize)
                .add_assign(count.to_owned());
        }
    }

    /// ONLY WORKS FOR STREET::TURN
    pub fn expectation(&self) -> f32 {
        self.weights
            .iter()
            .map(|(key, value)| (u64::from(key.clone()) as f32, value.clone() as f32))
            .map(|(x, y)| (x / Abstraction::N as f32, y / self.norm as f32))
            .map(|(x, y)| x * y)
            .sum()
    }
}

impl From<Observation> for Histogram {
    fn from(observation: Observation) -> Self {
        assert!(observation.street() == crate::cards::street::Street::Turn);
        observation
            .outnodes()
            .into_iter()
            .map(|obs| Abstraction::from(obs))
            .fold(Histogram::default(), |hist, abs| hist.witness(abs))
    }
}

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 1. interpret each key of the Histogram as probability
        let ref distribution = self
            .weights
            .iter()
            .map(|(key, value)| (u64::from(key.clone()) as f32, value.clone() as f32))
            .map(|(x, y)| (x / Abstraction::N as f32, y / self.norm as f32))
            .collect::<Vec<(f32, f32)>>();
        // 2. they should already be sorted bc BTreeMap
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
