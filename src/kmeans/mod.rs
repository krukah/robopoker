pub trait Point: Clone {}

pub trait KMeans<P>
where
    P: Point,
{
    fn loss(&self) -> f32;
    fn measure(&self, a: &P, b: &P) -> f32;
    fn average(&self, points: &[P]) -> P;
    fn dataset(&self) -> &[P; N];
    fn centers(&self) -> &[P; K];
    fn distances(&mut self) -> &mut [f32; N];
    fn neighbors(&mut self) -> &mut [usize; N]; // to what cluster is each point assigned
    fn densities(&mut self) -> &mut [usize; K]; // how many points are in each cluster
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

const N: usize = 1000;
const K: usize = 10;
