pub trait Point: Clone {}

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
    fn dataset(&self) -> &[P; N];
    /// mean dataset of Points
    fn centers(&self) -> &[P; K];
    /// get the distances
    fn distances(&mut self) -> &mut [f32; N];
    /// to what cluster is each point assigned
    fn neighbors(&mut self) -> &mut [usize; N];
    /// how many points are in each cluster
    fn densities(&mut self) -> &mut [usize; K];
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
