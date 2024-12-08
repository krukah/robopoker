#![allow(dead_code)]

use crate::cards::street::Street;
use crate::clustering::encoding::Encoder;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use rand::seq::SliceRandom;
use rand::Rng;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

type Neighbor = (usize, f32);

/// it's either we can take a (collected) Vec<P> and convert it into a P  (by &[P] nonetheless)
/// or we take (P, &P) -> P repeatedly and fold over it
pub trait Point: Clone {}

impl Point for Histogram {}

struct Layer {
    street: Street,
    metric: Metric,
    points: Vec<Histogram>,
    kmeans: Vec<Histogram>,
}

impl Layer {
    fn encoder(&self) -> Encoder {
        todo!("collet neighbors and turn into BTreeMap by zipping over ObservationIterator -> IsomorphismIterator")
    }
    /// accessors to internal kmeans (mtuating) and points (immutable)
    fn points(&self) -> &[Histogram] {
        &self.points
    }
    fn kmeans(&self) -> &[Histogram] {
        &self.kmeans
    }
    fn street(&self) -> Street {
        self.street
    }

    fn running(&self) -> bool {
        todo!()
    }
    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    fn initial(&self) -> Vec<Histogram> {
        let n = self.street().n_isomorphisms();
        let k = self.street().k();
        match self.street() {
            Street::Pref => {
                assert!(k == n, "lossless abstraction on preflop");
                self.points().to_vec()
            }
            _ => {
                let progress = crate::progress(k);
                let ref mut rng = rand::thread_rng();
                let mut initial = vec![];
                for _ in 0..k {
                    let sample = self.sample(rng, &initial);
                    initial.push(sample);
                    progress.inc(1);
                }
                progress.finish();
                initial
            }
        }
    }

    fn sample<R: Rng>(&self, rng: &mut R, kmeans: &[Histogram]) -> Histogram {
        match kmeans.len() {
            0 => self.points().choose(rng).unwrap().clone(), // uniform
            _ => self.kmeans().choose(rng).unwrap().clone(), // square weights
        }
    }
    fn cluster(&mut self) {
        let ref mut initial = self.initial();
        std::mem::swap(self.replace(), initial);
        while self.running() {
            let ref mut kmeans = self.iterate();
            std::mem::swap(self.replace(), kmeans);
        }
    }
    fn replace(&mut self) -> &mut Vec<Histogram> {
        &mut self.kmeans
    }
    fn iterate(&self) -> Vec<Histogram> {
        let k = self.street().k();
        let means = vec![Histogram::default(); k];
        self.nearests()
            .into_iter()
            .zip(self.points())
            .fold(means, |mut m, ((i, _), p)| {
                m[i].absorb(p);
                m
            })
    }
    fn nearests(&self) -> Vec<Neighbor> {
        self.points()
            .par_iter()
            .map(|h| self.neighbor(h))
            .collect::<Vec<Neighbor>>()
            .try_into()
            .expect("constant size N")
    }
    fn neighbor(&self, h: &Histogram) -> Neighbor {
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(i, k)| (i, self.metric.emd(h, k)))
            .min_by(|(_, dx), (_, dy)| dx.partial_cmp(dy).unwrap())
            .expect("find nearest neighbor")
            .into()
    }
}

pub enum Initialization {
    Random,              // chooses random points from dataset
    Spaced,              // chooses evenly spaced points from dataset
    FullPlusPlus,        // weights every point inverse distance to the nearest centroid
    MiniPlusPlus(usize), // weights batch point inverse distance to the nearest centroid
}

pub enum Termination {
    Iterations(usize),
    Convergent(usize, f32),
}
