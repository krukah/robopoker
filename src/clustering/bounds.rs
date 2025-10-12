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
}
