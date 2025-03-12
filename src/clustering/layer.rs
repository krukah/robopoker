use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::lookup::Lookup;
use super::metric::Metric;
use super::pair::Pair;
use super::transitions::Decomp;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::isomorphisms::IsomorphismIterator;
use crate::cards::street::Street;
use crate::Energy;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use std::collections::BTreeMap;

type Neighbor = (usize, f32);

pub struct Layer {
    street: Street,
    metric: Metric,
    points: Vec<Histogram>, // positioned by Isomorphism
    kmeans: Vec<Histogram>, // positioned by K-means abstraction
}

impl Layer {
    /// reference to the all points up to isomorphism
    fn points(&self) -> &Vec<Histogram> /* N */ {
        &self.points
    }
    /// reference to the current kmeans centorid histograms
    fn kmeans(&self) -> &Vec<Histogram> /* K */ {
        &self.kmeans
    }

    /// all-in-one entry point for learning the kmeans abstraction and
    /// writing to disk in pgcopy
    #[cfg(feature = "native")]
    pub fn learn() {
        use crate::save::upload::Table;
        Street::all()
            .into_iter()
            .rev()
            .filter(|&&s| !Self::done(s))
            .map(|&s| Self::grow(s).save())
            .count();
    }

    /// primary clustering algorithm loop
    #[cfg(feature = "native")]
    fn cluster(mut self) -> Self {
        log::info!("{:<32}{:<32}", "initialize  kmeans", self.street());
        let ref mut init = self.init();
        let ref mut last = self.kmeans;
        std::mem::swap(init, last);
        log::info!("{:<32}{:<32}", "clustering  kmeans", self.street());
        let t = self.street().t();
        let progress = crate::progress(t);
        for _ in 0..t {
            let ref mut next = self.next();
            let ref mut last = self.kmeans;
            std::mem::swap(next, last);
            progress.inc(1);
        }
        progress.finish();
        println!();
        self
    }

    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    #[cfg(feature = "native")]
    fn init(&self) -> Vec<Histogram> /* K */ {
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        use std::hash::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        // don't do any abstraction on preflop
        let k = self.street().k();
        let n = self.points().len();
        if self.street() == Street::Pref {
            assert!(n == k);
            return self.points().clone();
        }
        // deterministic pseudo-random clustering
        let ref mut hasher = DefaultHasher::default();
        self.street().hash(hasher);
        let ref mut rng = SmallRng::seed_from_u64(hasher.finish());
        // kmeans++ initialization
        let progress = crate::progress(k * n);
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
                .inspect(|_| progress.inc(1))
                .collect::<Vec<Energy>>()
                .iter()
                .zip(potentials.iter())
                .map(|(d0, d1)| Energy::min(*d0, *d1))
                .collect::<Vec<Energy>>();
        }
        progress.finish();
        println!();
        histograms
    }

    /// calculates the next step of the kmeans iteration by
    /// determining K * N optimal transport calculations and
    /// taking the nearest neighbor
    #[cfg(feature = "native")]
    fn next(&self) -> Vec<Histogram> /* K */ {
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        let k = self.street().k();
        let mut loss = 0f32;
        let mut centroids = vec![Histogram::default(); k];
        // assign points to nearest neighbors
        for (point, (neighbor, distance)) in self
            .points()
            .par_iter()
            .map(|h| (h, self.neighborhood(h)))
            .collect::<Vec<_>>()
            .into_iter()
        {
            loss = loss + distance * distance;
            centroids
                .get_mut(neighbor)
                .expect("index from neighbor calculation")
                .absorb(point);
        }
        log::debug!(
            "{:<32}{:<32}",
            "abstraction cluster RMS error",
            (loss / self.points().len() as f32).sqrt()
        );
        centroids
    }

    /// in ObsIterator order, get a mapping of
    /// Isomorphism -> Abstraction
    #[cfg(feature = "native")]
    fn lookup(&self) -> Lookup {
        log::info!("{:<32}{:<32}", "calculating lookup", self.street());
        use crate::save::upload::Table;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        let street = self.street();
        match street {
            Street::Pref | Street::Rive => Lookup::grow(street),
            Street::Flop | Street::Turn => self
                .points()
                .par_iter()
                .map(|h| self.neighborhood(h))
                .collect::<Vec<Neighbor>>()
                .into_iter()
                .map(|(k, _)| self.abstraction(k))
                .zip(IsomorphismIterator::from(street))
                .map(|(abs, iso)| (iso, abs))
                .collect::<BTreeMap<Isomorphism, Abstraction>>()
                .into(),
        }
    }

    /// wrawpper for distance metric calculations
    fn emd(&self, x: &Histogram, y: &Histogram) -> Energy {
        self.metric.emd(x, y)
    }
    /// because we have fixed-order Abstractions that are determined by
    /// street and K-index, we should encapsulate the self.street depenency
    fn abstraction(&self, i: usize) -> Abstraction {
        Abstraction::from((self.street(), i))
    }
    /// calculates nearest neighbor and separation distance for a Histogram
    fn neighborhood(&self, x: &Histogram) -> Neighbor {
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(k, h)| (k, self.emd(x, h)))
            .min_by(|(_, dx), (_, dy)| dx.partial_cmp(dy).unwrap())
            .expect("find nearest neighbor")
            .into()
    }

    /// reference to current street
    fn street(&self) -> Street {
        self.street
    }
    /// take outer triangular product of current learned kmeans
    /// Histograms, using whatever is stored as the future metric
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
    /// in AbsIterator order, get a mapping of
    /// Abstraction -> Histogram
    /// end-of-recurse call
    fn decomp(&self) -> Decomp {
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

#[cfg(feature = "native")]
impl crate::save::upload::Table for Layer {
    fn done(street: Street) -> bool {
        Lookup::done(street) && Decomp::done(street) && Metric::done(street)
    }
    fn save(&self) {
        self.metric().save();
        self.lookup().save();
        self.decomp().save();
    }
    fn grow(street: Street) -> Self {
        let layer = match street {
            Street::Rive => Self {
                street,
                kmeans: Vec::default(),
                points: Vec::default(),
                metric: Metric::default(),
            },
            _ => Self {
                street,
                kmeans: Vec::default(),
                points: Lookup::load(street.next()).projections(),
                metric: Metric::load(street.next()),
            },
        };
        layer.cluster()
    }

    fn name() -> String {
        unimplemented!()
    }
    fn copy() -> String {
        unimplemented!()
    }
    fn load(_: Street) -> Self {
        unimplemented!()
    }
    fn creates() -> String {
        unimplemented!()
    }
    fn indices() -> String {
        unimplemented!()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
    fn sources() -> Vec<String> {
        unimplemented!()
    }
}
