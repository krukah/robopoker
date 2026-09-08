//! What the database already knows about a bucket, before any cards are read.
use kicker::*;
use pokerkit::*;

/// The scalar facts SQL can state about one abstraction without looking at a
/// single card: how many isomorphisms landed in it, what fraction of its
/// street that is, and the first two moments of the equity distribution the
/// clustering actually optimized over.
///
/// `equity` and `spread` are read off the bucket's own centroid — the
/// `transitions` mass over next-street buckets weighted by their equity — so
/// they describe the object k-means clustered, not a card-level estimate.
#[derive(Copy, Clone, Debug)]
pub struct Census {
    abs: Abstraction,
    population: usize,
    share: Probability,
    equity: Probability,
    spread: Probability,
}

impl Census {
    pub fn abs(&self) -> Abstraction {
        self.abs
    }

    pub fn population(&self) -> usize {
        self.population
    }

    pub fn share(&self) -> Probability {
        self.share
    }

    pub fn equity(&self) -> Probability {
        self.equity
    }

    pub fn spread(&self) -> Probability {
        self.spread
    }

    /// A uniform sample of dense member positions — every position when the
    /// bucket is smaller than the sample. Positions are the dense per-bucket
    /// index `lloyd` writes alongside each isomorphism, so a sample is an
    /// index seek rather than a scan.
    ///
    /// Seeded on the bucket id, the way `lloyd` seeds k-means: two runs over
    /// the same clustering draw the same members and therefore derive the same
    /// name. An unseeded sampler makes a bucket's name drift across runs by a
    /// few frequency points, which is enough to flip a threshold and rewrite
    /// the label for no reason at all.
    pub fn positions(&self, size: usize) -> Vec<i32> {
        let mut positions = rand::seq::index::sample(&mut self.rng(), self.population, size.min(self.population))
            .into_iter()
            .map(|i| i as i32)
            .collect::<Vec<i32>>();
        positions.sort_unstable();
        positions
    }

    fn rng(&self) -> rand::rngs::SmallRng {
        rand::SeedableRng::seed_from_u64(u64::from(u16::from(self.abs)))
    }
}

impl From<(Abstraction, usize, Probability, Probability, Probability)> for Census {
    fn from(
        (abs, population, share, equity, spread): (Abstraction, usize, Probability, Probability, Probability),
    ) -> Self {
        Self {
            abs,
            population,
            share,
            equity,
            spread,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deuce::Street;

    #[test]
    fn positions_are_the_same_every_run() {
        let census = Census::from((Abstraction::from((Street::Flop, 7)), 10_000, 0.01, 0.5, 0.1));
        assert_eq!(census.positions(32), census.positions(32));
    }

    #[test]
    fn positions_differ_between_buckets() {
        let one = Census::from((Abstraction::from((Street::Flop, 7)), 10_000, 0.01, 0.5, 0.1));
        let two = Census::from((Abstraction::from((Street::Flop, 8)), 10_000, 0.01, 0.5, 0.1));
        assert_ne!(one.positions(32), two.positions(32));
    }

    #[test]
    fn a_small_bucket_is_sampled_whole() {
        let census = Census::from((Abstraction::from((Street::Flop, 7)), 12, 0.01, 0.5, 0.1));
        assert_eq!((0..12).collect::<Vec<i32>>(), census.positions(128));
    }
}
