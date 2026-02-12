/// Per-point metadata for Elkan's accelerated k-means algorithm.
///
/// Stores distance bounds that enable triangle inequality pruning,
/// dramatically reducing the number of expensive EMD computations.
/// Each point maintains bounds to all K centroids plus its current assignment.
///
/// # Algorithm (Elkan 2003)
///
/// The key insight: if we know d(x, c) ≤ u and d(c, c') ≤ 2u,
/// then d(x, c') cannot be less than d(x, c), so we skip computing it.
///
/// # Fields
///
/// - `j` — Index of currently assigned centroid (c(x) in paper)
/// - `lower` — Lower bounds l(x, c) for each centroid c
/// - `error` — Upper bound u(x) on distance to assigned centroid
/// - `stale` — Whether upper bound needs refreshing after centroid drift
#[derive(Debug, Clone)]
pub struct Bounds<const K: usize> {
    /// Currently assigned centroid index.
    j: usize,
    /// Lower bounds on distance to each centroid.
    lower: [f32; K],
    /// Upper bound on distance to assigned centroid.
    error: f32,
    /// Whether the upper bound is potentially stale.
    stale: bool,
}

impl<const K: usize> Bounds<K> {
    /// Currently assigned centroid index.
    pub fn j(&self) -> usize {
        self.j
    }
    /// Upper bound on distance to assigned centroid.
    pub fn u(&self) -> f32 {
        self.error
    }
    /// Whether the upper bound may be outdated.
    pub fn stale(&self) -> bool {
        self.stale
    }
    /// Gets lower bound for centroid j.
    fn lower(&self, j: usize) -> f32 {
        unsafe { *self.lower.get_unchecked(j) }
    }
    /// Sets lower bound for centroid j.
    fn set_lower(&mut self, j: usize, value: f32) {
        unsafe { *self.lower.get_unchecked_mut(j) = value }
    }
    /// Checks if centroid j could be closer than current assignment.
    ///
    /// Returns true (needs checking) if all triangle inequality filters fail:
    /// 1. j ≠ c(x) — not currently assigned
    /// 2. u(x) > l(x,j) — upper bound exceeds lower bound
    /// 3. u(x) > d(c(x),j)/2 — upper bound exceeds half inter-centroid distance
    pub fn has_shifted(&self, pairs: &[[f32; K]; K], j: usize) -> bool {
        unsafe {
            self.j() != j
                && self.u() > self.lower(j)
                && self.u() > 0.5 * pairs.get_unchecked(self.j()).get_unchecked(j)
        }
    }
    /// Checks if this point can skip Step 3 entirely.
    /// True when u(x) ≤ s(c(x)) where s(c) = min_{c'≠c} d(c,c')/2.
    pub fn can_exclude(&self, midpoints: &[f32; K]) -> bool {
        unsafe { self.u() <= *midpoints.get_unchecked(self.j()) }
    }
    /// Updates bounds after centroids move (Steps 5-6).
    /// Lowers are decreased by movement; upper is increased.
    pub fn update(&mut self, movements: &[f32; K]) {
        self.lower
            .iter_mut()
            .zip(movements.iter())
            .for_each(|(lower, movement)| *lower = (*lower - movement).max(0.0));
        unsafe { self.error += movements.get_unchecked(self.j()) }
        self.stale = true;
    }
    /// Refreshes upper bound by computing actual distance.
    pub fn refresh(&mut self, distance: f32) {
        self.set_lower(self.j(), distance);
        self.error = distance;
        self.stale = false;
    }
    /// Records distance to centroid j, reassigning if closer.
    pub fn witness(&mut self, distance: f32, j: usize) {
        self.set_lower(j, distance);
        if distance < self.u() {
            self.j = j;
            self.error = distance;
        }
    }

    /// Direct assignment (for naive k-means without bound tracking).
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
