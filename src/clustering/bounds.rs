/// Helper struct for the Elkan 2003 algorithm (Triangle-Inequality accelerated
/// version of Kmeans) to make it easier to pass along metadata about each
/// point into the function at each iteration.
///
/// Specifically, each instance of this struct contains "Carr[ied]...
/// information" between k-means iterations for a specific point in
/// `ClusterArg`'s `points` field. (Most notably: upper and lower distance
/// bounds to help avoid significant #s of redundant distance calculations.)
///
/// See Elkan (2003) for more information.
#[derive(Debug, Clone)]
pub struct Bound {
    /// The index into self.kmeans for the currently assigned
    /// centroid "nearest neighbor" (i.e. c(x) in the paper) for this
    /// specifed point.
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

    /// Elkan 2003 Step 3 filter: determines if bounds for centroid j should be updated
    /// Returns true if all three triangle inequality conditions are met:
    /// 1. j != c(x) - not currently assigned to this centroid
    /// 2. u(x) > l(x,j) - upper bound exceeds lower bound to j
    /// 3. u(x) > (1/2) * d(c(x), j) - upper bound exceeds half distance between centroids
    pub fn needs_update(&self, j: usize, pairs: &[Vec<f32>]) -> bool {
        self.j() != j && self.error > self.lower[j] && self.u() > 0.5 * pairs[self.j()][j]
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

    pub fn refresh(&mut self, distance: f32) -> f32 {
        let j = self.j();
        self.lower[j] = distance;
        self.error = distance;
        self.stale = false;
        distance
    }

    /// Get current upper bound, refreshing if stale
    fn get_current_upper<F>(&mut self, d: &F) -> f32
    where
        F: Fn(usize) -> f32,
    {
        if self.stale {
            self.refresh(d(self.j()))
        } else {
            self.error
        }
    }

    /// Check if we should compute actual distance to centroid j based on triangle inequality
    fn should_compute_distance(&self, j: usize, upper: f32, pairwises: &[Vec<f32>]) -> bool {
        upper > self.lower[j] || upper > 0.5 * pairwises[self.j()][j]
    }

    /// Compute distance to centroid j and potentially reassign if closer
    fn compute_and_maybe_reassign<F>(&mut self, j: usize, upper: f32, d: &F)
    where
        F: Fn(usize) -> f32,
    {
        let radius = d(j);
        self.lower[j] = radius;
        if radius < upper {
            self.j = j;
            self.error = radius;
        }
    }

    /// Update bounds for candidate centroid j (Elkan Step 3)
    pub fn update<F>(&mut self, j: usize, pairwises: &[Vec<f32>], d: F)
    where
        F: Fn(usize) -> f32,
    {
        let upper = self.get_current_upper(&d);
        if self.should_compute_distance(j, upper, pairwises) {
            self.compute_and_maybe_reassign(j, upper, &d);
        }
    }

    /// this is only used in the naive implementation of bounds generation,
    /// so we don't keep all the information that we have to with Elkan optimizaion.
    pub fn assign(&mut self, j: usize, distance: f32) {
        self.j = j;
        self.error = distance;
    }
}
