/// Helper struct for the Elkan 2003 algorithm (Triangle-Inequality accelerated
/// version of Kmeans) to make it easier to pass along metadata about each
/// point into the function at each iteration.
///
/// Specifically, each instance of this struct contains "Carried...
/// information" between k-means iterations for a specific point in
/// the Elkan::points() field. (Most notably: upper and lower distance
/// bounds to help avoid significant #s of redundant distance calculations.)
///
/// See Elkan (2003) for more information.
#[derive(Debug, Clone)]
pub struct Bounds<const K: usize> {
    /// The index into self.kmeans for the currently assigned
    /// centroid "nearest neighbor" (i.e. c(x) in the paper) for this
    /// specified point.
    j: usize,
    /// Lower bounds on the distance from this point to each centroid c
    /// (l(x,c) in the paper).
    lower: [f32; K],
    /// Distance to currently assigned centroid
    error: f32,
    stale: bool,
}

impl<const K: usize> Bounds<K> {
    pub fn j(&self) -> usize {
        self.j
    }
    pub fn u(&self) -> f32 {
        self.error
    }
    pub fn stale(&self) -> bool {
        self.stale
    }

    /// Elkan 2003 Step 3 filter: determines if bounds for centroid j should be updated
    /// Returns true if all three triangle inequality conditions are met:
    /// 1. j != c(x) - not currently assigned to this centroid
    /// 2. u(x) > l(x,j) - upper bound exceeds lower bound to j
    /// 3. u(x) > (1/2) * d(c(x), j) - upper bound exceeds half distance between centroids
    pub fn has_shifted(&self, pairs: &[[f32; K]; K], j: usize) -> bool {
        self.j() != j && self.u() > self.lower[j] && self.u() > 0.5 * pairs[self.j()][j]
    }

    /// Check if this point can be excluded from Step 3 processing
    /// Returns true if u(x) <= s(c(x)) where s(c(x)) is the midpoint
    pub fn can_exclude(&self, midpoints: &[f32; K]) -> bool {
        self.u() <= midpoints[self.j()]
    }

    /// Mutate bounds in place given centroid movements (Elkan Steps 5-6)
    pub fn update(&mut self, movements: &[f32; K]) {
        self.lower
            .iter_mut()
            .zip(movements.iter())
            .for_each(|(lower, movement)| *lower = (*lower - movement).max(0.0));
        self.error += movements[self.j()];
        self.stale = true;
    }

    pub fn refresh(&mut self, distance: f32) {
        let j = self.j();
        self.lower[j] = distance;
        self.error = distance;
        self.stale = false;
    }

    /// Try to reassign to centroid j if distance is closer than current assignment
    pub fn witness(&mut self, distance: f32, j: usize) {
        self.lower[j] = distance;
        if distance < self.u() {
            self.j = j;
            self.error = distance;
        }
    }

    /// this is only used in the naive implementation of bounds generation,
    /// so we don't keep all the information that we have to with Elkan optimizaion.
    pub fn assign(&mut self, distance: f32, j: usize) {
        self.j = j;
        self.error = distance;
    }
}

impl<const K: usize> Default for Bounds<K> {
    fn default() -> Self {
        Self {
            j: 0,
            lower: [0.0; K],
            error: 0.0,
            stale: false,
        }
    }
}

impl<const K: usize> From<(usize, f32)> for Bounds<K> {
    fn from((j, upper): (usize, f32)) -> Self {
        Self {
            j,
            lower: [0.0; K],
            error: upper,
            stale: false,
        }
    }
}
