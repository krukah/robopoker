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
pub struct Bound {
    /// The index into self.kmeans for the currently assigned
    /// centroid "nearest neighbor" (i.e. c(x) in the paper) for this
    /// specified point.
    pub j: usize,
    /// Lower bounds on the distance from this point to each centroid c
    /// (l(x,c) in the paper).
    pub lower: Vec<f32>,
    /// Distance to currently assigned centroid
    pub error: f32,
    pub stale: bool,
}

impl Bound {
    pub fn j(&self) -> usize {
        self.j
    }
    pub fn k(&self) -> usize {
        self.lower.len()
    }
    pub fn u(&self) -> f32 {
        self.error
    }
    pub fn new(j: usize, k: usize, upper: f32) -> Self {
        Self {
            j,
            lower: vec![0.0; k],
            error: upper,
            stale: false,
        }
    }

    pub fn stale(&self) -> bool {
        self.stale
    }

    /// Elkan 2003 Step 3 filter: determines if bounds for centroid j should be updated
    /// Returns true if all three triangle inequality conditions are met:
    /// 1. j != c(x) - not currently assigned to this centroid
    /// 2. u(x) > l(x,j) - upper bound exceeds lower bound to j
    /// 3. u(x) > (1/2) * d(c(x), j) - upper bound exceeds half distance between centroids
    pub fn moved(&self, pairs: &[Vec<f32>], j: usize) -> bool {
        self.j() != j
            && self.u() > self.lower.get(j).cloned().expect("k bounds")
            && self.u() > 0.5 * pairs[self.j()][j]
    }

    /// Check if this point can be excluded from Step 3 processing
    /// Returns true if u(x) <= s(c(x)) where s(c(x)) is the midpoint
    pub fn can_exclude(&self, midpoints: &[f32]) -> bool {
        self.u() <= midpoints[self.j()]
    }

    pub fn shift(self, movements: &[f32]) -> Self {
        self.update_lower(movements).update_upper(movements)
    }

    /// Update lower bounds given centroid movements (Elkan Step 5)
    fn update_lower(mut self, movements: &[f32]) -> Self {
        self.lower = self
            .lower
            .iter()
            .zip(movements)
            .map(|(lower, movement)| (lower - movement).max(0.0))
            .collect();
        self
    }

    /// Update upper bound given centroid movements (Elkan Step 6)
    fn update_upper(mut self, movements: &[f32]) -> Self {
        self.error += movements[self.j()];
        self.stale = true;
        self
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
    pub fn assign(&mut self, j: usize, distance: f32) {
        self.j = j;
        self.error = distance;
    }
}
