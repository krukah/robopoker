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
pub struct Bounds {
    /// The index into self.kmeans for the currently assigned
    /// centroid "nearest neighbor" (i.e. c(x) in the paper) for this
    /// specifed point.
    pub j: usize,
    /// Lower bounds on the distance from this point to each centroid c
    /// (l(x,c) in the paper).
    ///
    /// Is k in length, where k is the number of centroids in the k-means
    /// clustering. Each value inside the vector must correspond to the
    /// same-indexed **centroid** (not point!) in the Layer.
    pub lower: Vec<f32>,
    /// The upper bound on the distance from this point to its currently
    /// assigned centroid (u(x) in the paper).
    pub upper: f32,
    /// Whether the upper_bound is out-of-date and needs a 'refresh'(r(x) from
    /// the paper).
    pub stale: bool,
}

impl Bounds {
    pub fn new(assigned_centroid_idx: usize, k: usize, upper_bound: f32) -> Self {
        Self {
            j: assigned_centroid_idx,
            lower: vec![0.0; k],
            upper: upper_bound,
            stale: false,
        }
    }

    /// Elkan 2003 Step 3 filter: determines if bounds for centroid j should be updated
    /// Returns true if all three triangle inequality conditions are met:
    /// 1. j != c(x) - not currently assigned to this centroid
    /// 2. u(x) > l(x,j) - upper bound exceeds lower bound to j
    /// 3. u(x) > (1/2) * d(c(x), j) - upper bound exceeds half distance between centroids
    pub fn should_update(&self, j: usize, pairwises: &[Vec<f32>]) -> bool {
        j != self.j
            && self.upper > self.lower[j]
            && self.upper > 0.5 * pairwises[self.j][j]
    }

    /// Check if this point can be excluded from Step 3 processing
    /// Returns true if u(x) <= s(c(x)) where s(c(x)) is the midpoint
    pub fn can_exclude(&self, midpoints: &[f32]) -> bool {
        self.upper <= midpoints[self.j]
    }

    /// Update lower bounds given centroid movements (Elkan Step 5)
    pub fn update_lower(mut self, movements: &[f32]) -> Self {
        self.lower = self
            .lower
            .iter()
            .zip(movements)
            .map(|(lower, movement)| (lower - movement).max(0.0))
            .collect();
        self
    }

    /// Update upper bound given centroid movements (Elkan Step 6)
    pub fn update_upper(mut self, movements: &[f32]) -> Self {
        self.upper += movements[self.j];
        self.stale = true;
        self
    }

    /// Update bounds for candidate centroid j (Elkan Step 3)
    pub fn update<F>(&mut self, j: usize, pairwises: &[Vec<f32>], distance_to_center: F)
    where
        F: Fn(usize) -> f32,
    {
        let upper = if self.stale {
            let d = distance_to_center(self.j);
            self.lower[self.j] = d;
            self.upper = d;
            self.stale = false;
            d
        } else {
            self.upper
        };
        if upper > self.lower[j] || upper > 0.5 * pairwises[self.j][j] {
            let radius = distance_to_center(j);
            self.lower[j] = radius;
            if radius < upper {
                self.j = j;
                self.upper = radius;
            }
        }
    }
}
