use crate::cards::street::Street;
use crate::clustering::encoding::Encoder;
use crate::clustering::histogram::Histogram;
use crate::clustering::layer::Layer;
use crate::clustering::metric::Metric;
use crate::Utility;

pub trait Point: Clone {}
impl Point for Histogram {}

type Neighbor = (usize, f32);
type Populace = usize;

struct StreetKmeans<const K: usize, const N: usize> {
    street: Street,
    lookup: Encoder,
    metric: Metric,
    kmeans: [Histogram; K],
    points: [Histogram; N],
}

impl KMeans<Histogram> for StreetKmeans<K, N> {
    fn loss(&self) -> f32 {
        todo!()
    }

    fn measure(&self, a: &Histogram, b: &Histogram) -> f32 {
        todo!()
    }

    fn average(&self, points: &[Histogram]) -> Histogram {
        todo!()
    }

    fn points(&self) -> &[Histogram; N] {
        todo!()
    }

    fn kmeans(&self) -> &[Histogram; K] {
        todo!()
    }

    fn neighbors(&self) -> [Neighbor; N] {
        todo!()
    }
}

/// number of points in dataset
const N: usize = 1000;
/// number of clusters
const K: usize = 10;

pub trait KMeans<P>
where
    P: Point,
{
    /// do some ::fold on the dataset to accumulate loss metric
    fn loss(&self) -> f32;
    /// abstract distance metric
    fn measure(&self, a: &P, b: &P) -> f32;
    /// average a collection of points
    fn average(&self, points: &[P]) -> P;
    /// full dataset of Points
    fn points(&self) -> &[P; N];
    /// mean dataset of Points
    fn kmeans(&self) -> &[P; K];
    /// freshly calculated nearest neighbors + distances
    fn neighbors(&self) -> [Neighbor; N];
}

impl<P> Iterator for dyn KMeans<P>
where
    P: Point,
{
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        // do the inner of Layer::cluster_kmeans() loop
        // calculate neighbors &self.neighbors()
        // calculate densities &self.densities()
        // check against stopping rule(s)
        Some(self.loss())
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
