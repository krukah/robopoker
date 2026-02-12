use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;

/// A distribution over abstraction buckets for a specific street.
///
/// Histograms are the core data type for clustering: each poker hand
/// isomorphism maps to a histogram describing its distribution over
/// the next street's abstraction buckets. For example, a turn hand's
/// histogram shows how often it transitions into each river equity bucket.
///
/// # Stack Allocation
///
/// Uses a tagged enum over street-specific [`Bins`] arrays to avoid
/// heap allocation. The enum variant determines the array size at
/// compile time (e.g., `BinsRive` for river histograms).
///
/// # Usage in Clustering
///
/// 1. Compute histograms for all isomorphisms at a given street
/// 2. Cluster histograms using k-means with EMD distance
/// 3. Each cluster centroid becomes an abstraction bucket
/// 4. The histogram→bucket mapping becomes the [`Lookup`] table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Histogram {
    Pref(BinsPref),
    Flop(BinsFlop),
    Turn(BinsTurn),
    Rive(BinsRive),
}

impl Histogram {
    /// Creates an empty histogram for the given street.
    pub const fn new(street: Street) -> Self {
        Self::empty(street)
    }
    /// Creates an empty histogram with zero weight.
    pub const fn empty(street: Street) -> Self {
        match street {
            Street::Pref => Histogram::Pref(Bins::new(street)),
            Street::Flop => Histogram::Flop(Bins::new(street)),
            Street::Turn => Histogram::Turn(Bins::new(street)),
            Street::Rive => Histogram::Rive(Bins::new(street)),
        }
    }
    /// Sets the count for a specific abstraction bucket.
    pub fn set(&mut self, abs: Abstraction, count: usize) {
        match self {
            Histogram::Pref(b) => b.set(abs, count),
            Histogram::Flop(b) => b.set(abs, count),
            Histogram::Turn(b) => b.set(abs, count),
            Histogram::Rive(b) => b.set(abs, count),
        }
    }
    /// the weight of a given Abstraction. returns 0 if the Abstraction was never witnessed.
    pub fn density(&self, x: &Abstraction) -> Probability {
        match self {
            Histogram::Pref(b) => b.density(x),
            Histogram::Flop(b) => b.density(x),
            Histogram::Turn(b) => b.density(x),
            Histogram::Rive(b) => b.density(x),
        }
    }
    /// all witnessed Abstractions in the support
    pub fn support(&self) -> impl Iterator<Item = Abstraction> + '_ {
        match self {
            Histogram::Pref(b) => IterWrap::Pref(b.support()),
            Histogram::Flop(b) => IterWrap::Flop(b.support()),
            Histogram::Turn(b) => IterWrap::Turn(b.support()),
            Histogram::Rive(b) => IterWrap::Rive(b.support()),
        }
    }
    /// size of the support
    pub fn n(&self) -> usize {
        match self {
            Histogram::Pref(b) => b.n(),
            Histogram::Flop(b) => b.n(),
            Histogram::Turn(b) => b.n(),
            Histogram::Rive(b) => b.n(),
        }
    }
    /// the street of abstractions contained in this histogram
    pub fn street(&self) -> Street {
        match self {
            Histogram::Pref(b) => b.street(),
            Histogram::Flop(b) => b.street(),
            Histogram::Turn(b) => b.street(),
            Histogram::Rive(b) => b.street(),
        }
    }
    /// insert the Abstraction into our support,
    /// incrementing its local weight,
    /// incrementing our global norm.
    pub fn increment(mut self, abstraction: Abstraction) -> Self {
        match &mut self {
            Histogram::Pref(b) => b.increment(abstraction),
            Histogram::Flop(b) => b.increment(abstraction),
            Histogram::Turn(b) => b.increment(abstraction),
            Histogram::Rive(b) => b.increment(abstraction),
        }
        self
    }
    /// absorb the other histogram into this one.
    pub fn absorb(mut self, other: &Self) -> Self {
        self.merge(other);
        self
    }
    pub fn merge(&mut self, other: &Self) {
        match (self, other) {
            (Histogram::Pref(a), Histogram::Pref(b)) => a.merge(b),
            (Histogram::Flop(a), Histogram::Flop(b)) => a.merge(b),
            (Histogram::Turn(a), Histogram::Turn(b)) => a.merge(b),
            (Histogram::Rive(a), Histogram::Rive(b)) => a.merge(b),
            _ => panic!("mismatched histogram streets in engulf"),
        }
    }
    /// it is useful in EMD calculation
    /// to know if we're dealing with ::Equity or ::Random
    /// Abstraction variants, so we expose this method to
    /// infer the type of Abstraction contained by this Histogram.
    pub fn peek(&self) -> Abstraction {
        match self {
            Histogram::Pref(b) => b.peek(),
            Histogram::Flop(b) => b.peek(),
            Histogram::Turn(b) => b.peek(),
            Histogram::Rive(b) => b.peek(),
        }
    }
    /// exhaustive calculation of all
    /// possible Rivers and Showdowns,
    /// naive to strategy of course.
    pub fn equity(&self) -> Probability {
        match self {
            Histogram::Pref(b) => b.equity(),
            Histogram::Flop(b) => b.equity(),
            Histogram::Turn(b) => b.equity(),
            Histogram::Rive(b) => b.equity(),
        }
    }
    /// this yields the posterior equity distribution
    /// at Street::Turn.
    /// this is the only street we explicitly can calculate
    /// the Probability of transitioning into a Probability
    ///     Probability -> Probability
    /// vs  Probability -> Abstraction
    /// hence a distribution over showdown equities.
    pub fn pdf(&self) -> Vec<(Probability, Probability)> {
        match self {
            Histogram::Pref(b) => b.pdf(),
            Histogram::Flop(b) => b.pdf(),
            Histogram::Turn(b) => b.pdf(),
            Histogram::Rive(b) => b.pdf(),
        }
    }
    /// owned vector of Abstractions and their densities
    /// sorted by density in descending order (most likely first)
    pub fn distribution(&self) -> Vec<(Abstraction, Probability)> {
        match self {
            Histogram::Pref(b) => b.distribution(),
            Histogram::Flop(b) => b.distribution(),
            Histogram::Turn(b) => b.distribution(),
            Histogram::Rive(b) => b.distribution(),
        }
    }
}

/// Helper enum to unify different support iterators
enum IterWrap<A, B, C, D> {
    Pref(A),
    Flop(B),
    Turn(C),
    Rive(D),
}

impl<A, B, C, D> Iterator for IterWrap<A, B, C, D>
where
    A: Iterator<Item = Abstraction>,
    B: Iterator<Item = Abstraction>,
    C: Iterator<Item = Abstraction>,
    D: Iterator<Item = Abstraction>,
{
    type Item = Abstraction;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterWrap::Pref(i) => i.next(),
            IterWrap::Flop(i) => i.next(),
            IterWrap::Turn(i) => i.next(),
            IterWrap::Rive(i) => i.next(),
        }
    }
}

impl From<Observation> for Histogram {
    fn from(ref turn: Observation) -> Self {
        debug_assert!(turn.street() == Street::Turn);
        turn.children()
            .map(|river| river.equity())
            .map(Abstraction::from)
            .fold(Histogram::empty(Street::Rive), Histogram::increment)
    }
}

impl From<Vec<Abstraction>> for Histogram {
    fn from(a: Vec<Abstraction>) -> Self {
        let street = a.first().unwrap().street();
        a.into_iter()
            .fold(Histogram::empty(street), Histogram::increment)
    }
}

impl Density for Histogram {
    type Support = ClusterAbs;
    fn density(&self, x: &Self::Support) -> f32 {
        Histogram::density(self, x)
    }
    fn support(&self) -> impl Iterator<Item = Self::Support> {
        Histogram::support(self).map(ClusterAbs::from)
    }
}

impl Arbitrary for Histogram {
    fn random() -> Self {
        const S: usize = 16;
        const M: usize = 64;
        (0..)
            .map(|_| Abstraction::random())
            .filter(|a| a.street() == Street::Flop)
            .take(S)
            .collect::<Vec<_>>()
            .into_iter()
            .cycle()
            .filter(|_| rand::random::<bool>())
            .take(M)
            .fold(Histogram::empty(Street::Flop), Histogram::increment)
    }
}

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_assert!(matches!(self.peek().street(), Street::Rive));
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
