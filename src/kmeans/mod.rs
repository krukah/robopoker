pub trait KMeans<P> {
    fn k(&self) -> usize;
    fn dataset(&self) -> &[P; N];
    fn cluster(&self) -> &[P; K];
    fn measure(&self, a: &P, b: &P) -> f32;
    fn average(&self, cluster: &[P]) -> P;
    fn assignments(&self) -> &[usize; N]; // to what cluster is each point assigned
    fn frequencies(&self) -> &[usize; K]; // how many points are in each cluster
}
const N: usize = 1000;
const K: usize = 10;
