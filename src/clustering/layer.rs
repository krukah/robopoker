use super::bounds::Bound;
use super::histogram::Histogram;
use super::kmeans::Elkan;
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

type Neighbor = (usize, f32);

pub struct Layer {
    street: Street,
    metric: Metric,
    points: Vec<Histogram>, // positioned by Isomorphism
    kmeans: Vec<Histogram>, // positioned by K-means abstraction
    bounds: Vec<Bound>,     // empty - only used for Elkan acceleration
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
    pub fn init(&self) -> (Vec<Histogram>, Vec<Bound>) {
        (self.init_kmeans(), self.init_bounds())
    }

    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    fn init_kmeans(&self) -> Vec<Histogram> {
        use rand::distr::weighted::WeightedIndex;
        use rand::distr::Distribution;
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

    fn init_bounds(&self) -> Vec<Bound> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        (0..self.n())
            .into_par_iter()
            .map(|i| self.neighbor(i))
            .map(|(j, dist)| Bound::new(j, self.k(), dist))
            .collect::<Vec<_>>()
    }

    /// in ObsIterator order, get a mapping of
    /// Isomorphism -> Abstraction
    fn lookup(&self) -> Lookup {
        log::info!("{:<32}{:<32}", "calculating lookup", self.street());
        use crate::save::disk::Disk;
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        let street = self.street();
        match street {
            Street::Pref | Street::Rive => Lookup::grow(street),
            Street::Flop | Street::Turn => (0..self.n())
                .into_par_iter()
                .map(|i| self.neighbor(i))
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
    fn decomp(&self) -> Shadow {
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

impl Elkan for Layer {
    type P = Histogram;
    fn t(&self) -> usize {
        self.street().t()
    }
    fn k(&self) -> usize {
        self.street().k()
    }
    fn n(&self) -> usize {
        self.street().n_isomorphisms()
    }
    fn dataset(&self) -> &Vec<Histogram> {
        &self.points
    }
    fn centers(&self) -> &Vec<Histogram> {
        &self.kmeans
    }
    fn boundaries(&self) -> &Vec<Bound> {
        &self.bounds
    }
    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric.emd(h1, h2)
    }
    fn step(&mut self) {
        let (centers, boundaries) = self.next();
        self.bounds = boundaries;
        self.kmeans = centers;
    }
}

impl crate::save::disk::Disk for Layer {
    fn name() -> String {
        format!(
            "{:<16}{:<16}{:<16}",
            Lookup::name(),
            Shadow::name(),
            Metric::name()
        )
    }
    fn done(street: Street) -> bool {
        Lookup::done(street) && Shadow::done(street) && Metric::done(street)
    }
    fn save(&self) {
        self.metric().save();
        self.lookup().save();
        self.decomp().save();
    }
    fn grow(street: Street) -> Self {
        let mut layer = Self::load(street);
        let (centers, boundaries) = layer.init();
        layer.kmeans = centers;
        layer.bounds = boundaries;
        for _ in 0..layer.t() {
            layer.step();
        }
        layer
    }
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
