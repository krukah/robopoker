//! K-means clustering layer for poker hand abstraction.
//!
//! This module implements a single clustering layer that maps poker hand isomorphisms
//! to abstract buckets using the k-means algorithm with Elkan acceleration.

use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
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
/// 2. A `Future` transition model mapping abstractions to next-street distributions
/// 3. A `Metric` defining distances between learned abstractions
pub struct Layer<const K: usize, const N: usize> {
    /// The betting street this layer represents
    street: Street,
    /// Distance metric for computing EMD between abstractions in the next street
    metric: Box<Metric>,
    /// Learned k-means cluster centroids, indexed by abstraction (K total)
    kmeans: Box<[Histogram; K]>,
    /// All poker hand histograms, indexed by isomorphism order (N total)
    points: Box<[Histogram; N]>,
    /// Distance bounds for each point, used by Elkan acceleration (not persisted)
    bounds: Box<[Bounds<K>; N]>,
}

impl<const K: usize, const N: usize> Layer<K, N> {
    /// Returns the betting street for this layer.
    fn street(&self) -> Street {
        self.street
    }

    /// Constructs an `Abstraction` from this layer's street and a cluster index.
    fn abstraction(&self, i: usize) -> Abstraction {
        Abstraction::from((self.street(), i))
    }
}

impl<const K: usize, const N: usize> Layer<K, N> {
    /// Builds a lookup table mapping each isomorphism to its nearest cluster abstraction.
    fn lookup(&self) -> Lookup
    where
        Self: Elkan<K, N>,
    {
        log::info!("{:<32}{:<32}", "calculating lookup", self.street());
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        match self.street() {
            Street::Pref | Street::Rive => Lookup::grow(self.street()),
            Street::Flop | Street::Turn => (0..N)
                .into_par_iter()
                .map(|i| self.neighbor(i))
                .collect::<Vec<(usize, f32)>>()
                .into_iter()
                .map(|(k, _)| self.abstraction(k))
                .zip(IsomorphismIterator::from(self.street()))
                .map(|(abs, iso)| (iso, abs))
                .collect::<BTreeMap<Isomorphism, Abstraction>>()
                .into(),
        }
    }

    /// Computes pairwise distances between all learned cluster centroids.
    fn metric(&self) -> Metric {
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

    /// Builds the transition future hand mapping abstractions to their centroid histograms.
    fn future(&self) -> Future {
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
impl<const K: usize, const N: usize> Elkan<K, N> for Layer<K, N> {
    type P = Histogram;

    fn t(&self) -> usize {
        self.street().t()
    }

    fn points(&self) -> &[Histogram; N] {
        &self.points
    }

    fn kmeans(&self) -> &[Histogram; K] {
        &self.kmeans
    }

    fn bounds(&self) -> &[Bounds<K>; N] {
        &self.bounds
    }

    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric.emd(h1, h2)
    }

    fn init_kmeans(&self) -> [Histogram; K] {
        use rand::SeedableRng;
        use rand::distr::Distribution;
        use rand::distr::weighted::WeightedIndex;
        use rand::rngs::SmallRng;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        use std::hash::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        // don't do any abstraction on preflop or river
        if matches!(self.street(), Street::Pref | Street::Rive) {
            debug_assert!(N == K);
            return std::array::from_fn(|i| self.points()[i]);
        }
        // deterministic pseudo-random clustering
        let ref mut hasher = DefaultHasher::default();
        self.street().hash(hasher);
        let ref mut rng = SmallRng::seed_from_u64(hasher.finish());
        // kmeans++ initialization
        let mut potentials = vec![1.; N];
        let mut histograms = Vec::with_capacity(K);
        while histograms.len() < K {
            let i = WeightedIndex::new(potentials.iter())
                .expect("valid weights array")
                .sample(rng);
            let x = self.points()[i];
            histograms.push(x);
            potentials[i] = 0.;
            potentials = self
                .points()
                .par_iter()
                .map(|h| self.distance(&x, &h))
                .map(|p| p * p)
                .collect::<Vec<Energy>>()
                .iter()
                .zip(potentials.iter())
                .map(|(d0, d1)| Energy::min(*d0, *d1))
                .collect::<Vec<Energy>>();
        }
        histograms.try_into().expect("K")
    }
}

#[cfg(feature = "database")]
impl<const K: usize, const N: usize> Layer<K, N> {
    /// Internal clustering implementation for a specific K, N.
    pub async fn cluster(street: Street, client: &tokio_postgres::Client) -> Artifacts {
        log::info!("{:<32}{:<32}", "kmeans hydrating", street);
        let mut layer = Self::build(street, client).await;
        log::info!("{:<32}{:<32}", "kmeans initializing", street);
        layer.kmeans = Box::new(layer.init_kmeans());
        log::info!("{:<32}{:<32}", "kmeans bounding", street);
        layer.bounds = layer.init_bounds();
        log::info!("{:<32}{:<32}", "kmeans iterating", street);
        let new = vec![Bounds::default(); N].try_into().expect("N");
        let ref mut old = layer.bounds;
        let ref mut old = std::mem::replace(old, new);
        for i in 0..layer.t() {
            layer.kmeans = Box::new(layer.step_elkan(old));
            log::debug!("{:3}", i);
        }
        let ref mut new = layer.bounds;
        std::mem::swap(new, old);
        Artifacts {
            lookup: layer.lookup(),
            metric: layer.metric(),
            future: layer.future(),
        }
    }
    /// Build layer dependencies from postgres (not disk).
    async fn build(street: Street, client: &tokio_postgres::Client) -> Self {
        if street == Street::Rive {
            Self {
                street,
                metric: Box::new(Metric::default()),
                kmeans: Box::new(std::array::from_fn(|_| Histogram::empty(Street::Rive))),
                bounds: vec![Bounds::default(); N].try_into().expect("N"),
                points: vec![Histogram::empty(Street::Rive); N]
                    .try_into()
                    .expect("N"),
            }
        } else {
            Self {
                street,
                metric: Box::new(Metric::from_street(client, street.next()).await),
                kmeans: Box::new(std::array::from_fn(|_| Histogram::empty(street.next()))),
                bounds: vec![Bounds::default(); N].try_into().expect("N"),
                points: Lookup::from_street(client, street.next())
                    .await
                    .projections()
                    .try_into()
                    .expect("projections.len() == N"),
            }
        }
    }
}
