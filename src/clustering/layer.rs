//! K-means clustering layer for poker hand abstraction.
//!
//! This module implements a single clustering layer that maps poker hand isomorphisms
//! to abstract buckets using the k-means algorithm with Elkan acceleration.

use super::bounds::Bound;
use super::elkan::Elkan;
use super::histogram::Histogram;
use super::lookup::Lookup;
use super::metric::Metric;
use super::pair::Pair;
use super::transitions::Shadow;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::isomorphisms::IsomorphismIterator;
use crate::cards::street::Street;
use crate::gameplay::abstraction::Abstraction;
use crate::Energy;
use std::collections::BTreeMap;

/// A clustering layer that maps poker hand isomorphisms to abstract buckets.
///
/// Each layer corresponds to a single betting street and maintains:
/// - The full dataset of hand histograms (one per isomorphism)
/// - K-means cluster centroids learned via the Elkan algorithm
/// - Distance bounds for acceleration during clustering
///
/// The layer produces three artifacts:
/// 1. A `Lookup` table mapping isomorphisms to abstractions
/// 2. A `Shadow` transition model mapping abstractions to next-street distributions
/// 3. A `Metric` defining distances between learned abstractions
pub struct Layer {
    /// The betting street this layer represents
    street: Street,
    /// Distance metric for computing EMD between abstractions in the next street
    metric: Metric,
    /// All poker hand histograms, indexed by isomorphism order (N total)
    points: Vec<Histogram>,
    /// Learned k-means cluster centroids, indexed by abstraction (K total)
    kmeans: Vec<Histogram>,
    /// Distance bounds for each point, used by Elkan acceleration (not persisted)
    bounds: Vec<Bound>,
}

impl Layer {
    /// Returns a reference to all data points (N total).
    ///
    /// These are the poker hand histograms representing the complete dataset,
    /// with one histogram per isomorphism class.
    fn points(&self) -> &Vec<Histogram> {
        &self.points
    }

    /// Returns a reference to the current k-means cluster centroids (K total).
    ///
    /// Each centroid is a histogram representing the center of a learned abstraction bucket.
    fn kmeans(&self) -> &Vec<Histogram> {
        &self.kmeans
    }

    /// Entry point for learning k-means abstractions across all streets and persisting to disk.
    ///
    /// This function:
    /// 1. Logs which streets already have computed abstractions
    /// 2. Computes and saves abstractions for streets that need them
    ///
    /// Streets are processed in reverse order (River → Turn → Flop → Preflop)
    /// to ensure dependencies are available when needed.
    pub fn learn() {
        use crate::save::disk::Disk;
        Street::all()
            .into_iter()
            .rev()
            .filter(|&&s| Self::done(s))
            .for_each(|s| log::info!("{:<32}{:<16}{:<32}", "using kmeans layer", s, Self::name()));
        Street::all()
            .into_iter()
            .rev()
            .filter(|&&s| !Self::done(s))
            .map(|&s| Self::grow(s).save())
            .count();
    }
}

impl Layer {
    /// Builds a lookup table mapping each isomorphism to its nearest cluster abstraction.
    ///
    /// For preflop and river, returns a trivial 1:1 mapping since no clustering is performed.
    /// For flop and turn, assigns each hand isomorphism to its nearest learned cluster centroid
    /// using parallel computation.
    ///
    /// Returns a `Lookup` structure in `IsomorphismIterator` order.
    fn lookup(&self) -> Lookup {
        // @parallelizable
        log::info!("{:<32}{:<32}", "calculating lookup", self.street());
        use crate::save::disk::Disk;
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        let street = self.street();
        match street {
            Street::Pref | Street::Rive => Lookup::grow(street),
            Street::Flop | Street::Turn => {
                let progress = crate::progress(self.n());
                let result = (0..self.n())
                    .into_par_iter()
                    .map(|i| self.neighbor(i))
                    .inspect(|_| progress.inc(1))
                    .collect::<Vec<(usize, f32)>>()
                    .into_iter()
                    .map(|(k, _)| self.abstraction(k))
                    .zip(IsomorphismIterator::from(street))
                    .map(|(abs, iso)| (iso, abs))
                    .collect::<BTreeMap<Isomorphism, Abstraction>>()
                    .into();
                progress.finish();
                result
            }
        }
    }

    /// Computes the earth mover's distance (EMD) between two histograms.
    ///
    /// This is a thin wrapper around the metric's EMD calculation.
    fn emd(&self, x: &Histogram, y: &Histogram) -> Energy {
        self.metric.emd(x, y)
    }

    /// Constructs an `Abstraction` from this layer's street and a cluster index.
    ///
    /// Abstractions have a fixed order determined by (street, k-index),
    /// so this method encapsulates the street dependency.
    fn abstraction(&self, i: usize) -> Abstraction {
        Abstraction::from((self.street(), i))
    }

    /// Returns the betting street for this layer.
    fn street(&self) -> Street {
        self.street
    }

    /// Computes pairwise distances between all learned cluster centroids.
    ///
    /// Builds a `Metric` containing the symmetric average EMD for each pair of abstractions.
    /// This metric is used by the previous street when computing distances between its histograms.
    ///
    /// Only computes the lower triangular portion to avoid redundant work.
    fn metric(&self) -> Metric {
        // @parallelizable
        log::info!("{:<32}{:<32}", "calculating metric", self.street());
        let mut metric = BTreeMap::new();
        for (i, x) in self.kmeans.iter().enumerate() {
            for (j, y) in self.kmeans.iter().enumerate() {
                if i > j {
                    let ref a = self.abstraction(i);
                    let ref b = self.abstraction(j);
                    let index = Pair::from((a, b));
                    let distance = self.metric.emd(x, y) + self.metric.emd(y, x);
                    let distance = distance / 2.;
                    metric.insert(index, distance);
                }
            }
        }
        Metric::from(metric)
    }

    /// Builds the transition shadow mapping abstractions to their centroid histograms.
    ///
    /// Returns a `Shadow` structure in `AbstractionIterator` order that maps each abstraction
    /// on this street to its distribution over next-street abstractions.
    ///
    /// This is the terminal case in the recursive abstraction refinement process.
    fn decomp(&self) -> Shadow {
        // @parallelizable
        log::info!("{:<32}{:<32}", "calculating transitions", self.street());
        self.kmeans()
            .iter()
            .cloned()
            .enumerate()
            .map(|(k, centroid)| (self.abstraction(k), centroid))
            .collect::<BTreeMap<Abstraction, Histogram>>()
            .into()
    }
}

/// Elkan k-means implementation for clustering poker hand abstractions.
///
/// This trait implementation provides the Elkan-accelerated k-means algorithm
/// with distance bounds to avoid redundant distance calculations.
impl Elkan for Layer {
    /// The data point type is a histogram over next-street abstractions.
    type P = Histogram;

    /// Returns the number of k-means iterations to perform for this street.
    fn t(&self) -> usize {
        self.street().t()
    }

    /// Returns the target number of clusters (K) for this street.
    fn k(&self) -> usize {
        self.street().k()
    }

    /// Returns the complete dataset of points to be clustered.
    ///
    /// # Panics
    /// Panics if the number of points doesn't match the expected count.
    fn dataset(&self) -> &Vec<Histogram> {
        assert!(self.points.len() == self.n());
        &self.points
    }

    /// Returns the current k-means cluster centroids.
    ///
    /// # Panics
    /// Panics if the number of centroids doesn't match K.
    fn kmeans(&self) -> &Vec<Histogram> {
        assert!(self.kmeans.len() == self.k());
        &self.kmeans
    }

    /// Returns the distance bounds for Elkan acceleration.
    ///
    /// # Panics
    /// Panics if the number of bounds doesn't match the number of data points.
    fn bounds(&self) -> &Vec<Bound> {
        assert!(self.bounds.len() == self.n());
        &self.bounds
    }

    /// Computes the distance between two histograms using EMD.
    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric.emd(h1, h2)
    }

    /// Initializes k-means centroids using the k-means++ algorithm.
    ///
    /// The algorithm:
    /// 1. Selects the first centroid uniformly at random from the dataset
    /// 2. For each subsequent centroid, selects a point with probability proportional
    ///    to the squared distance from its nearest existing centroid
    /// 3. Repeats until K centroids are selected
    ///
    /// For preflop and river streets, returns the full dataset unchanged since
    /// no clustering is performed (N = K).
    ///
    /// Uses deterministic seeding based on the street for reproducibility.
    fn init_kmeans(&self) -> Vec<Histogram> {
        // @parallelizable
        use rand::distr::weighted::WeightedIndex;
        use rand::distr::Distribution;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        use std::hash::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        // don't do any abstraction on preflop or river
        let k = self.street().k();
        let n = self.points().len();
        if matches!(self.street(), Street::Pref | Street::Rive) {
            assert!(n == k);
            return self.points().clone();
        }
        // deterministic pseudo-random clustering
        let ref mut hasher = DefaultHasher::default();
        self.street().hash(hasher);
        let ref mut rng = SmallRng::seed_from_u64(hasher.finish());
        // kmeans++ initialization
        let mut potentials = vec![1.; n];
        let mut histograms = Vec::new();
        while histograms.len() < k {
            let i = WeightedIndex::new(potentials.iter())
                .expect("valid weights array")
                .sample(rng);
            let x = self
                .points()
                .get(i)
                .expect("sharing index with outer layer");
            histograms.push(x.clone());
            potentials[i] = 0.;
            potentials = self
                .points()
                .par_iter()
                .map(|h| self.emd(x, h))
                .map(|p| p * p)
                .collect::<Vec<Energy>>()
                .iter()
                .zip(potentials.iter())
                .map(|(d0, d1)| Energy::min(*d0, *d1))
                .collect::<Vec<Energy>>();
        }
        histograms
    }
}

/// Disk persistence implementation for saving and loading clustering layers.
///
/// A layer produces and persists three artifacts:
/// - Lookup table (isomorphism → abstraction mappings)
/// - Shadow transitions (abstraction → histogram distributions)
/// - Metric (pairwise distances between abstractions)
impl crate::save::disk::Disk for Layer {
    /// Returns a formatted name combining the three artifact types.
    fn name() -> String {
        format!(
            "{:<16}{:<16}{:<16}",
            Lookup::name(),
            Shadow::name(),
            Metric::name()
        )
    }

    /// Checks if all three artifacts have been persisted for the given street.
    fn done(street: Street) -> bool {
        Lookup::done(street) && Shadow::done(street) && Metric::done(street)
    }

    /// Persists all three artifacts (metric, lookup, transitions) to disk.
    fn save(&self) {
        self.metric().save();
        self.lookup().save();
        self.decomp().save();
    }

    /// Learns k-means clusters for the given street and returns the trained layer.
    ///
    /// The process:
    /// 1. Loads the layer (including next-street data as the metric and points)
    /// 2. Initializes centroids using k-means++
    /// 3. Initializes distance bounds for Elkan acceleration
    /// 4. Runs k-means iterations to convergence
    ///
    /// # Parameters
    /// - `street`: The betting street to cluster
    fn grow(street: Street) -> Self {
        let mut layer = Self::load(street);
        log::info!("{:<32}{:<32}", "initializing kmeans", street);
        layer.kmeans = layer.init_kmeans();
        layer.bounds = layer.init_bounds();
        log::info!("{:<32}{:<32}", "clustering   kmeans", street);
        let progress = crate::progress(street.t());
        for _ in 0..layer.t() {
            let (kmeans, bounds) = layer.next_eklan();
            layer.kmeans = kmeans;
            layer.bounds = bounds;
            progress.inc(1);
        }
        progress.finish();
        layer
    }

    /// Loads a layer for the given street from persisted next-street data.
    ///
    /// For the river, creates an empty layer since there is no next street.
    /// For other streets, loads:
    /// - Points: Histogram projections from the next street's lookup table
    /// - Metric: Distance metric from the next street
    ///
    /// # Parameters
    /// - `street`: The betting street to load
    fn load(street: Street) -> Self {
        match street {
            Street::Rive => Self {
                street,
                bounds: Vec::default(),
                kmeans: Vec::default(),
                points: Vec::default(),
                metric: Metric::default(),
            },
            _ => Self {
                street,
                bounds: Vec::default(),
                kmeans: Vec::default(),
                points: Lookup::load(street.next()).projections(),
                metric: Metric::load(street.next()),
            },
        }
    }
}
