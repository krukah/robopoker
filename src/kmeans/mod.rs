use crate::cards::isomorphism::Isomorphism;
use crate::cards::isomorphisms::IsomorphismIterator;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;

type Neighbor = (usize, f32);

struct Layer {
    street: Street,
    metric: Metric,
    points: Vec<Histogram>,
    kmeans: Vec<Histogram>,
}

impl Layer {
    /// primary clustering algorithm loop
    fn cluster(&mut self) {
        let ref mut start = self.init();
        std::mem::swap(start, &mut self.kmeans);
        while true {
            let ref mut means = self.next();
            std::mem::swap(means, &mut self.kmeans);
        }
    }

    fn inner_street(&self) -> Street {
        self.street().prev()
    }
    fn inner_metric(&self) -> Metric {
        todo!("double for loop, outer product over self.kmeans()")
    }
    fn inner_points(&self) -> Vec<Histogram> {
        // get owned instance of BTreeMap<I, A> as Lookup
        use rayon::iter::IntoParallelIterator;
        IsomorphismIterator::from(self.street().prev())
            .collect::<Vec<Isomorphism>>()
            .into_par_iter()
            .map(|inner| self.encode.projection(inner))
    }
    fn inner_kmeans(&self) -> Vec<Histogram> {
        vec![]
    }

    /// reference to current street
    fn street(&self) -> &Street {
        &self.street
    }
    /// distance metric for Abstractions of the current layer
    fn metric(&self) -> &Metric {
        &self.metric
    }
    /// reference to the observed points
    fn points(&self) -> &Vec<Histogram> /* N */ {
        &self.points
    }
    /// reference to the current kmeans centorid histograms
    fn kmeans(&self) -> &Vec<Histogram> /* K */ {
        &self.kmeans
    }

    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    fn init(&self) -> Vec<Histogram> /* K */ {
        let n = self.street().n_isomorphisms();
        let k = self.street().k();
        todo!()
    }
    /// calculates the next step of the kmeans iteration by
    /// determining K * N optimal transport calculations and
    /// taking the nearest neighbor
    fn next(&self) -> Vec<Histogram> /* K */ {
        use rayon::iter::IntoParallelRefIterator;
        //? check for empty centroids??
        let kmeans = vec![Histogram::default(); self.street().k()];
        self.points()
            .par_iter()
            .map(|h| (h, self.neighbored(h)))
            .collect::<Vec<(&Histogram, Neighbor)>>()
            .into_iter()
            .fold(kmeans, |mut kmeans, (hist, (mean, _))| {
                kmeans.get_mut(mean).expect("bounds").absorb(hist);
                kmeans
            })
    }

    /// because we have fixed-order Abstractions that are determined by
    /// street and K-index, we should encapsulate the self.street depenency
    fn abstracted(&self, i: usize) -> Abstraction {
        Abstraction::from((self.street().clone(), i))
    }
    /// calculates nearest neighbor and separation distance for a Histogram
    fn neighbored(&self, h: &Histogram) -> Neighbor {
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(i, k)| (i, self.metric().emd(h, k)))
            .min_by(|(_, dx), (_, dy)| dx.partial_cmp(dy).unwrap())
            .expect("find nearest neighbor")
            .into()
    }

    /// in ObsIterator order, get a mapping of
    /// Isomorphism -> Abstraction
    fn embeddings(&self) -> BTreeMap<Isomorphism, Abstraction> {
        use rayon::iter::IntoParallelRefIterator;
        self.points()
            .par_iter()
            .map(|h| self.neighbored(h))
            .collect::<Vec<Neighbor>>()
            .into_iter()
            .map(|(k, _)| self.abstracted(k))
            .zip(IsomorphismIterator::from(self.street().clone()))
            .map(|(abs, iso)| (iso, abs))
            .collect::<BTreeMap<Isomorphism, Abstraction>>()
    }
    /// in AbsIterator order, get a mapping of
    /// Abstraction -> Histogram
    fn histograms(&self) -> BTreeMap<Abstraction, Histogram> {
        self.kmeans()
            .iter()
            .cloned()
            .enumerate()
            .map(|(k, mean)| (self.abstracted(k), mean))
            .collect::<BTreeMap<Abstraction, Histogram>>()
    }
}

trait Lookup {
    fn street(&self) -> Street;
    fn future(&self, iso: &Isomorphism) -> Histogram;
    fn lookup(&self, obs: &Observation) -> Abstraction;
    fn assign(&mut self, abs: &Abstraction, iso: &Isomorphism);
}
