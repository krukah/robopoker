use super::histogram::Histogram;
use super::lookup::Lookup;
use super::metric::Metric;
use super::pair::Pair;
use super::transitions::Decomp;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::isomorphisms::IsomorphismIterator;
use crate::cards::street::Street;
use crate::gameplay::abstraction::Abstraction;
use crate::Energy;
use indicatif::ProgressIterator;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::SystemTime;

type Neighbor = (usize, f32);

pub struct Layer {
    street: Street,
    metric: Metric,
    points: Vec<Histogram>, // positioned by Isomorphism
    kmeans: Vec<Histogram>, // positioned by K-means abstraction
}

// "Carr[ied]... information" between k-means iterations for a specific point
// in self.points. See Elkan 2003 for more details.
//
// Used to accelerate k-means clustering via the paper's Triangle Inequality
// (abrv. 'TriIneq' here) based optimized algorithm.
//
// NOTE: Includes some additional fields besides _just_ the bounds. (E.g. a
// field to help lookup the currently assigned centroid for the point).
#[derive(Debug, Clone)]
struct TriIneqBounds {
    // The index into self.kmeans for the currently assigned centroid "nearest
    // neighbor" (i.e. c(x) in the paper) for this specifed point.
    assigned_centroid_idx: usize,
    // Lower bounds on the distance from this point to each centroid c
    // (l(x,c) in the paper).
    // Is k in length, where k is the number of centroids in the k-means
    // clustering. Each value inside the vector must correspond to the
    // same-indexed **centroid** (not point!) in the Layer.
    lower_bounds: Vec<f32>,
    // The upper bound on the distance from this point to its currently
    // assinged centroid (u(x) in the paper).
    upper_bound: f32,
    // Whether the upper_bound is out-of-date and needs a 'refresh'
    // (r(x) from the paper).
    stale_upper_bound: bool,
}

impl Layer {
    /// reference to the all points up to isomorphism
    fn points(&self) -> &Vec<Histogram> /* N */ {
        &self.points
    }
    /// reference to the current kmeans centorid histograms
    fn kmeans(&self) -> &Vec<Histogram> /* K */ {
        &self.kmeans
    }

    /// all-in-one entry point for learning the kmeans abstraction and
    /// writing to disk in pgcopy
    pub fn learn() {
        use crate::save::disk::Disk;
        Street::all()
            .into_iter()
            .rev()
            .filter(|&&s| Self::done(s))
            .for_each(|s| log::info!("{:<32}{:<16}{:<32}", "using kmeans layer", s, Self::name()));
        Street::all()
            .into_iter()
            .rev()
            .filter(|&&s| !Self::done(s))
            .map(|&s| Self::grow(s).save())
            .count();
    }

    fn cluster(mut self) -> Self {
        log::info!("{:<32}{:<32}", "initialize  kmeans", self.street());
        let ref mut init = self.init();
        let ref mut last = self.kmeans;
        std::mem::swap(init, last);

        let k = self.street().k();
        if k == 0 {
            return self;
        }
        let t = self.street().t();

        // TODO: Replace this with something less hacky + controllable
        // programmatically from outside of this function.
        let triangle_accelerate_todo_replaceme = true;
        if !triangle_accelerate_todo_replaceme {
            log::info!(
                "{:<32}{:<32}",
                "clustering kmeans (unaccelerated)",
                self.street()
            );
            let progress = crate::progress(t);
            for _ in 0..t {
                // WARNING: If modifying this code, MAKE SURE THAT THE RMS VALUES ACTUALLY
                // DECREASE ON EACH ITERATION. (We've hit a bug in the past trying to fix this
                // where the code looked fine at casual glance, but in practice it wasn't
                // actually making progress past the first iteration.)
                let ref mut next = self.compute_next_kmeans();
                let ref mut last = self.kmeans;
                std::mem::swap(next, last);
                progress.inc(1);
            }
            progress.finish();
            println!();
        } else {
            // Use Triangle Inequality (TI) math to accelerate the K-means
            // clustering, as per Elkan (2003).

            log::debug!(
                "Initializing helpers for triangle-inequality (TI) accelerated of clustering"
            );
            // """
            // First, pick initial centers. Set the lower bound l(x,c) for each point x
            // and center c. Assign each x to its closest initial center c(x) =
            // argmin_c d(x,c), using Lemma 1 to avoid redundant distance
            // calculations. Each time d(x,c) is computed, set l(x,c) = d(x,c). Assign
            // upper bounds u(x) = min_c d(x,c).
            // """
            let mut ti_helpers: Vec<TriIneqBounds> = self
                // TODO: Double check we're not repeating the 'pick initial centers' work here twice.
                // (e.g. if we already did that during the init() above)
                .create_centroids_tri_ineq()
                .iter()
                .map(|nearest_neighbor| TriIneqBounds {
                    // "c(x)"'s index in self.kmeans()
                    assigned_centroid_idx: nearest_neighbor.0,
                    // "l(x,c)"
                    // "Set the lower bound l(x,c) = 0 for each point x and center c"
                    lower_bounds: vec![0.0; self.street().k()],
                    // "u(x)"
                    // "Assign upper bounds u(x) = min_c d(x,c)" (which by
                    //  definition is the distance of the nearest neighbor at
                    //  this point)
                    upper_bound: nearest_neighbor.1,
                    // "r(x)"
                    // (Not explicitly mentioned during the pre-step. But, we know that
                    // when starting out we literally _just_computed all the distances,
                    // so it should theoretically be safe to leave 'false' here.)
                    stale_upper_bound: false,
                })
                .collect::<Vec<_>>();
            log::debug!("Completed TI helper initialization.");

            log::info!(
                "{:<32}{:<32}",
                "clustering kmeans (*accelerated*)",
                self.street()
            );
            // TODO: Verify results are actually the same from here as in the
            // pre-existing, non-accelerated algorithm above. As per the paper:
            // """
            // We want the accelerated k-means algorithm to be usable wherever the
            // standard algorithm is used. Therefore, we need the accelerated
            // algorithm to satisfy three properties. First, it should be able to
            // start with any initial centers, so that all existing
            // initialization methods can continue to be used. Second, given the
            // same initial centers, it should al- ways produce exactly the same
            // final centers as the standard algorithm. Third, it should be able
            // to use any black-box distance metric, so it should not rely for
            // example on optimizations specific to Euclidean distance.
            //
            // Our algorithm in fact satisfies a condition stronger than the
            // second one above: after each iteration, it produces the same set
            // of center locations as the standard k-means method.
            // """
            // TODO: Add styling to progress bar
            for i in (0..t).progress() {
                log::debug!("{:<32}{:<32}", "Performing training iteration # ", i);
                let (ref mut next_kmeans, ref mut next_helpers) =
                    self.compute_next_kmeans_tri_ineq(&ti_helpers);

                let ref mut current_kmeans = self.kmeans;
                std::mem::swap(current_kmeans, next_kmeans);

                let ref mut current_helpers = ti_helpers;
                std::mem::swap(current_helpers, next_helpers)
            }
        }
        self
    }

    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    fn init(&self) -> Vec<Histogram> /* K */ {
        use rand::distr::weighted::WeightedIndex;
        use rand::distr::Distribution;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        use std::hash::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        // don't do any abstraction on preflop
        let k = self.street().k();
        let n = self.points().len();
        if self.street() == Street::Pref {
            assert!(n == k);
            return self.points().clone();
        }
        // deterministic pseudo-random clustering
        let ref mut hasher = DefaultHasher::default();
        self.street().hash(hasher);
        let ref mut rng = SmallRng::seed_from_u64(hasher.finish());
        // kmeans++ initialization
        let progress = crate::progress(k * n);
        let mut potentials = vec![1.; n];
        let mut histograms = Vec::new();
        while histograms.len() < k {
            let i = WeightedIndex::new(potentials.iter())
                .expect("valid weights array")
                .sample(rng);
            let x = self
                .points()
                .get(i)
                .expect("sharing index with outer layer");
            histograms.push(x.clone());
            potentials[i] = 0.;
            potentials = self
                .points()
                .par_iter()
                .map(|h| self.emd(x, h))
                .map(|p| p * p)
                .inspect(|_| progress.inc(1))
                .collect::<Vec<Energy>>()
                .iter()
                .zip(potentials.iter())
                .map(|(d0, d1)| Energy::min(*d0, *d1))
                .collect::<Vec<Energy>>();
        }
        progress.finish();
        println!();
        histograms
    }

    #[cfg(feature = "native")]
    /// calculates the next step of the kmeans iteration by
    /// determining K * N optimal transport calculations and
    /// taking the nearest neighbor
    fn compute_next_kmeans(&self) -> Vec<Histogram> /* K */ {
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        let k = self.street().k();
        let mut loss = 0f32;
        let mut centroids = vec![Histogram::default(); k];
        // assign points to nearest neighbors
        for (point, (neighbor, distance)) in self
            .points()
            .par_iter()
            .map(|h| (h, self.neighborhood(h)))
            .collect::<Vec<_>>()
            .into_iter()
        {
            loss = loss + distance * distance;
            centroids
                .get_mut(neighbor)
                .expect("index from neighbor calculation")
                .absorb(point);
        }
        log::debug!(
            "{:<32}{:<32}",
            "abstraction cluster RMS error",
            (loss / self.points().len() as f32).sqrt()
        );
        centroids
    }

    #[cfg(feature = "native")]
    /// Triangle(tri)-Inequality(ineq) accelerated version of kmeans.
    ///
    /// Calculates the next step of the kmeans iteration by efficiently
    /// computing (a subset of) K * N optimal transport calculations and
    /// taking the nearest neighbor, using triangle inequalities where
    /// possible to skip performing unnecessary calculations.
    ///
    /// In theory, the algorithm used here is guaranteed to produce the same
    /// results as the 'unaccelerated' kmeans at every iteration given the
    /// same set of inputs, while providing a massive speedup in most
    /// real-world situations.
    fn compute_next_kmeans_tri_ineq(
        &self,
        ti_helpers: &[TriIneqBounds],
    ) -> (
        Vec<Histogram>,     /* K centroids */
        Vec<TriIneqBounds>, /* Updated Triangle Inequality Helpers */
    ) {
        // TODO: panic if the length of ti_helpers doesn't match the length of
        // self.points

        use indicatif::ParallelProgressIterator;
        use rayon::iter::IndexedParallelIterator;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;

        let k = self.street().k();

        // TODO start tracking loss like in the other approach!
        // let mut loss = 0f32;

        // TODO: refactor / start using some things like this
        // let n = self.points().len();

        // ****
        // The following 7-step algorithm is taken from Elkan (2003). It uses
        // triangle inequalities to accelerate the k-means algorithm.
        // ****

        // *Step 1*: For all centers c and c', compute d(c,c'). For all
        //  centers c, compute s(c) = (1/2) min_{c'!=c} d(c, c')
        //
        // This means s effectively contains the 'distance to the midpoint
        // between this centroid and the closest other centroid' for each
        // centroid.
        log::debug!("{:<32}", " - Elkan Step 1");
        // Step 1 (first half): d(c, c') for all centers c and c'
        let centroid_to_centroid_distances: Vec<Vec<f32>> = self
            .kmeans()
            .iter()
            // Get all combinations [(c1,c1), (c1,c2), ... (c_k, c_k)] into
            // a simple 1-D vector to allow for easily parallelizing the emd
            // calculations.
            // TLDR: effectively just itertools.array_combinations().
            .flat_map(|c| self.kmeans().iter().map(move |c_prime| (c, c_prime)))
            .collect::<Vec<_>>()
            .par_iter()
            .map(|(center1, center2)| self.emd(center1, center2)) // 1-D vector with length k^2
            .collect::<Vec<f32>>()
            .chunks(k) // Separate into k-length chunks so we can get it into a 2-D vector
            .map(|chunked| chunked.to_vec())
            .collect();

        // Step 1 (second half): s(c) = (1/2) min_{c'!=c} d(c, c')
        // (i.e. the closest midpoint to another centroid besides itself)
        let per_centroid_distance_to_closest_midpoint: Vec<f32> = centroid_to_centroid_distances
            .iter()
            .enumerate()
            .map(|(i, distances_from_centroid_i)| {
                // TLDR reducing down each per-centroid row to 1/2 the minimum
                // distance to all centroids except itself
                distances_from_centroid_i
                    .iter()
                    .enumerate()
                    .filter(|(other_centroid_index, _distance)| *other_centroid_index != i)
                    .map(|(_other_centroid_index, distance)| distance * 0.5)
                    // Workaround for f32 not implementing Ord due to NaN
                    // being incomparable.
                    // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.min
                    .reduce(f32::min)
                    // ... TBD - might want to actually do something non-zero,
                    // seems like 0 could bite us if something "weird" were to
                    // happen here.
                    .unwrap_or(0.)
            })
            .collect();

        log::debug!("{:<32}", " - Elkan Step 2");

        // Step 2: "Identify all points x such that u(x) <= s(c(x)).", i.e.
        // where the upper bound for the opint is less than its closest
        // midpoint.
        //
        // See also from the paper: "Logically, step (2) is redundant ...
        // [but c]omputationally step (2) is beneficial because if it
        // eliminates a point x from further consideration, then comparing u
        // (x) to l(x,c) for every c separately is not necessary."
        let step_2_excluded_points: HashSet<usize> = ti_helpers
            .iter()
            .enumerate()
            .filter_map(|(x, helper)| {
                // Grab the s(c(x)) we computed earlier, i.e. passing c
                // (x) into s(c). So it's not the index of the point itself
                // that we should look up in s, but rather the index of
                // the _centroid to which the point x is currently assigned_.
                // Or in other words - the index of x's current "nearest
                // neighbor".
                let step1_s_of_c_of_x =
                    per_centroid_distance_to_closest_midpoint[helper.assigned_centroid_idx];
                if helper.upper_bound <= step1_s_of_c_of_x {
                    Some(x)
                } else {
                    None
                }
            })
            .collect();

        let mut step_3_working_points: HashMap<usize, (&Histogram, TriIneqBounds)> = self
            .points()
            .iter()
            .enumerate()
            .filter(|(point_i, _)| !step_2_excluded_points.contains(point_i))
            .map(|(point_i, point_h)| (point_i, (point_h, ti_helpers[point_i].clone())))
            .collect();

        // Step 3: For all remaining points x and centers c such that ...
        //
        // ** THIS HASHMAP WILL BE UPDATED IN PLACE VIA PARALLELIZED CODE
        // BELOW **
        //
        // See paper as follows:
        // "In step (3), each time d(x, c) is calculated for any x and c, its
        //  lower bound is updated by assigning l(x, c) = d(x, c). Similarly,
        //  u(x) is updated whenever c(x) is changed or d(x, c(x)) is
        //  computed.
        //
        // Note also: "When step (3) is implemented with nested loops, the
        // outer loop can be over x or over c. For efficiency ... the outer
        // loop should be over c since k << n typically, and the inner loop
        // should be replaced by vectorized code that operates on all
        // relevant x collectively."
        //
        // Note: technically we're not _quite_ doing what the paper says, i.e.
        // pure vectorized math. Instead we're having to settle for Rayon
        // parallelization on account of using Histograms and non-Euclidean
        // distances.
        log::debug!("{:<32}", " - Elkan Step 3");
        use rayon::prelude::*;
        for (center_c_idx, center_c) in self.kmeans().iter().enumerate() {
            step_3_working_points
                .par_iter_mut()
                // As far as we can tell, this progress bar doesn't negatively affect
                // performance. HOWEVER, it does mess with the other progress bar +
                // doesn't look tidy (due to being unstyled), so we should probably
                // come back and clean it up a bit.
                .progress_count(self.points().len().try_into().unwrap())
                // _point_i used later for step 4 lookups but unneeded when mutating here
                .for_each(|(_point_i, (point_h, helper))| {
                    // STEP 3 FILTERING: Apply all three filter conditions with early exits
                    // STEP 3.i: Skip if c == c(x) (point already assigned to this centroid)
                    // STEP 3.ii: Skip if u(x) <= l(x, c) (upper bound not greater than lower bound)
                    // STEP 3.iii: Skip if u(x) <= (1/2) * d(c(x), c)
                    // (i.e. upper bound not greater than half centroid distance)
                    if center_c_idx == helper.assigned_centroid_idx
                        || helper.upper_bound <= helper.lower_bounds[center_c_idx]
                        || helper.upper_bound
                            <= 0.5
                                * centroid_to_centroid_distances[helper.assigned_centroid_idx]
                                    [center_c_idx]
                    {
                        return;
                    }

                    // STEP 3.a: "If r(x) then compute d(x, c(x)) and assign r(x) = false.
                    //           Otherwise, d(x, c(x)) = u(x)."
                    let current_centroid_dist = if helper.stale_upper_bound {
                        let dist = self.emd(point_h, &self.kmeans()[helper.assigned_centroid_idx]);
                        // As discussed above: "each time d(x, c) is
                        // calculated for any x and c, its lower bound is
                        // updated by assigning l(x, c) = d(x, c)" and
                        // "u(x) is updated whenever c(x) is changed or d
                        //  (x, c(x)) is computed."
                        helper.upper_bound = dist; // Update u(x) in-place
                        helper.lower_bounds[helper.assigned_centroid_idx] = dist; // Update l(x, c(x)) in-place

                        // Step 3.a: If r(x) then compute d(x, c(x)) and
                        // assign r(x) = false. Otherwise, d(x, c(x)) = u
                        // (x).
                        helper.stale_upper_bound = false; // clear r (x) in-place
                        dist
                    } else {
                        // Use existing upper bound as d(x, c(x))
                        helper.upper_bound
                    };

                    // Step 3.b:
                    //  If d(x, c(x)) > l(x,c)
                    //  or d(x, c(x)) > (1/2) d(c(x), c)
                    // then:
                    //  Compute d(x,c)
                    //  If d(x,c) < d(x, c(x)) then assign c(x) = c
                    if current_centroid_dist > helper.lower_bounds[center_c_idx]
                        || current_centroid_dist
                            > 0.5
                                * centroid_to_centroid_distances[helper.assigned_centroid_idx]
                                    [center_c_idx]
                    {
                        //  ... "Compute d(x,c)"
                        let dist_to_center_c = self.emd(point_h, center_c);
                        // (As discussed above: "each time d(x, c) is calculated ...")
                        helper.lower_bounds[center_c_idx] = dist_to_center_c; // update l(x,c) in place

                        // ... If d(x,c) < d(x, c(x)) then assign c(x) = c
                        if dist_to_center_c < current_centroid_dist {
                            helper.assigned_centroid_idx = center_c_idx; // Reassign c(x) = c in-place

                            // As discussed above: "u(x) is updated whenever c
                            // (x) is changed or d(x, c(x)) is computed."
                            // Notably, ~2 lines up we computing d(x, c), but
                            // that's NOT the same as d(x, c(x)). So we only
                            // need to update upper bound if we actually made
                            // it into here.
                            helper.upper_bound = dist_to_center_c // update u (x) in place
                        }
                    }
                });
        }

        log::debug!("{:<32}", " - Elkan Step 4");
        // Merge the updated helper values back with the original vector we got
        // at the start of the function (which has entries for *all* points, not
        // just the ones bieng updated in step 3).
        let step_4_helpers: Vec<&TriIneqBounds> = ti_helpers
            .iter()
            .enumerate()
            .map(|(point_i, original_helper)| {
                if step_3_working_points.contains_key(&point_i) {
                    &(step_3_working_points[&point_i].1)
                } else {
                    original_helper
                }
            })
            .collect();

        // Step 4: For each center c, let m(c) be the mean of the points
        // assigned to c.
        //
        // This becomes the new centroid for the next step:
        // """
        // Step 4 computes the new lcoation fo each cluster center c.
        // Setting m(c) to be the mean of the points assigned to c is
        // appropriate when the distance metric in use is Euclidean distance.
        // Otherwise m(c) may be defined differently...
        // """
        //
        // Note also:
        // """
        // Step 4 computes the new location of each cluster center.
        // Setting m(c) to be the mean of the points assigned to is
        // appropriate when the distance metric in use is Euclidean
        // distance. Otherwise, may be defined differently. For
        // example, with k-medians the new center of each cluster is
        // a representative member of the cluster.
        // """
        //
        // In this case it's a little weird looking ('aborbing' histograms)
        // since we're using emd instead of Euclidean distance.
        let points_assigned_per_center: Vec<Vec<&Histogram>> = self
            .kmeans()
            .iter()
            .enumerate()
            .map(|(center_c_idx, _center_c)| {
                step_4_helpers
                    .iter()
                    .enumerate()
                    .filter(|(_point_i, helper)| helper.assigned_centroid_idx == center_c_idx)
                    .map(|(point_i, _)| &self.points()[point_i])
                    .collect()
            })
            .collect();

        let perform_extra_loss_calculations = true;
        if perform_extra_loss_calculations {
            log::debug!(
                "Performing {} otherwise-unnecessary emd computations to
                 calculate RMS error. This may slow down step 4!",
                self.points.len()
            );
        }
        let mut loss = 0f32;
        let mut rms_calculation_seconds = 0;

        let mut new_centroids: Vec<Histogram> = vec![];

        // Note: we could optionally parallelize this. In practice though, so long
        // as `perform_extra_loss_calculations` is false this is so fast that it's
        // not worth doing.
        for (centroid_i, points) in points_assigned_per_center.iter().enumerate() {
            let mut mean_of_assigned_points = points[0].clone();
            if points.is_empty() {
                // TODO: Figure out what to do for the centroid if there's no poitns assigned to it.
                log::error!("No points assigned to current centroid. This is currently an edge case we are unable to resolve; for more details see https://github.com/krukah/robopoker/issues/34#issuecomment-2860641178")
            }

            for point in points.iter().skip(1) {
                mean_of_assigned_points.absorb(point);
            }
            let next_centroid = mean_of_assigned_points;

            if perform_extra_loss_calculations {
                // NOTE: Calculating the error with the OLD center (to ensure
                // that this is consistent with the unaccelerated algorithm).
                let old_centroid = &(self.kmeans()[centroid_i]);
                // As mentioned above, this can be expensive; we add extra tracking
                // here to allow the user to more easily determine if it's worth
                // disabling or not.
                let now = SystemTime::now();
                for point in points.iter() {
                    let distance_point_to_prior_centroid = self.emd(&old_centroid, &point);
                    loss += distance_point_to_prior_centroid * distance_point_to_prior_centroid;
                }
                match now.elapsed() {
                    Ok(elapsed) => {
                        rms_calculation_seconds += elapsed.as_secs();
                    }
                    Err(e) => {
                        log::error!("Error tracking elapsed time for RMS calculations: {e:?}");
                    }
                }
            }


            new_centroids.push(next_centroid);
        }
        if perform_extra_loss_calculations {
            log::debug!(
                "{:<32}{:<32}",
                "abstraction cluster RMS error",
                (loss / self.points().len() as f32).sqrt()
            );
            // Arbitrarily setting a threshold for when it 'affects' performance;
            // anything under is clearly so small that the reminder message isn't
            // needed. (Arguably we could set this much higher too.)
            if rms_calculation_seconds > 2 {
                log::warn!(
                    "Calculating RMS error for debug logs required an extra {}
                    seconds (to perform {} otherwise-unnecessary distance
                    computations). Consider disabling to improve performance.",
                    rms_calculation_seconds,
                    self.points().len()
                );
            }
        }

        log::debug!("{:<32}", " - Elkan Step 5");
        // Step 5: Update lower bounds. From paper: ""
        // 5. For each point x and center c, assign
        //    l(x,c) = max{ l(x, c) - d(c, m(c)), 0 }
        // """
        //
        // Optimization: compute distance from each centroid to the its
        // replacement *one time* rather than inside the per-point loops of
        // steps 5 and 6.
        // (Note: this is highly differant than step 1! This isn't computing
        // each centroid to *all* the replacements, only *its* specific
        // replacement (i.e. the one with the same index). Hence it's a
        // Vec<f32> of length k, NOT a Vec<Vec<f32>>.
        //
        // TODO: double check that distance is equivalent in either direction.
        // (If not, would need to e.g. compute a Vec<(f32, f32)> instead.)
        let new_centroid_movements: Vec<f32> = new_centroids
            .par_iter()
            .zip(&self.kmeans)
            .map(|(old_center, new_center)| self.emd(old_center, new_center))
            .collect();

        let step_5_helpers: Vec<TriIneqBounds> = step_4_helpers
            .into_par_iter()
            .cloned()
            .map(|mut helper| {
                // Update lower_bounds in-place (sequential within each helper)
                // Note: looks up d(c, m(c)) from when we calculated it outside
                // the loop above.
                for (lower_bound, &centroid_movement) in
                    helper.lower_bounds.iter_mut().zip(&new_centroid_movements)
                {
                    *lower_bound = (*lower_bound - centroid_movement).max(0.0);
                }
                helper
            })
            .collect();

        log::debug!("{:<32}", " - Elkan Step 6");
        // Step 6: Update upper bounds. From paper: """
        // 6. For each point x, assign
        //    u(x) = u(x) + d(m(c(x)), c(x))
        //    r(x) = true
        // """
        // TODO refactor probably can get away with continuing to borrow here.
        // And/or do using a .map() inside in a .par_iter() etc.
        let mut step_6_helpers: Vec<TriIneqBounds> = step_5_helpers;
        for helper in &mut step_6_helpers {
            // u(x) = u(x) + d(m(c(x)), c(x))
            // TODO: VERIFY THAT d(m(c(x)), c(x)) = d(c(x), m(c(x))).
            // IF NOT THEN THIS WILL BE INCORRECT.
            let dist_center_and_new_center = &new_centroid_movements[helper.assigned_centroid_idx];
            helper.upper_bound += dist_center_and_new_center;
            // r(x) = true
            helper.stale_upper_bound = true;
        }

        // Form paper "[Compute] the new location of each cluster center",
        // i.e. Step 7:
        // "7. Replace each center c by m(c)"
        log::debug!("{:<32}", " - Elkan Step 7");
        (new_centroids, step_6_helpers)
    }

    /// in ObsIterator order, get a mapping of
    /// Isomorphism -> Abstraction
    fn lookup(&self) -> Lookup {
        log::info!("{:<32}{:<32}", "calculating lookup", self.street());
        use crate::save::disk::Disk;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        let street = self.street();
        match street {
            Street::Pref | Street::Rive => Lookup::grow(street),
            Street::Flop | Street::Turn => self
                .points()
                .par_iter()
                .map(|h| self.neighborhood(h))
                .collect::<Vec<Neighbor>>()
                .into_iter()
                .map(|(k, _)| self.abstraction(k))
                .zip(IsomorphismIterator::from(street))
                .map(|(abs, iso)| (iso, abs))
                .collect::<BTreeMap<Isomorphism, Abstraction>>()
                .into(),
        }
    }

    /// wrawpper for distance metric calculations
    fn emd(&self, x: &Histogram, y: &Histogram) -> Energy {
        self.metric.emd(x, y)
    }
    /// because we have fixed-order Abstractions that are determined by
    /// street and K-index, we should encapsulate the self.street depenency
    fn abstraction(&self, i: usize) -> Abstraction {
        Abstraction::from((self.street(), i))
    }
    /// calculates nearest neighbor and separation distance for a Histogram
    fn neighborhood(&self, x: &Histogram) -> Neighbor {
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(k, h)| (k, self.emd(x, h)))
            .min_by(|(_, dx), (_, dy)| dx.partial_cmp(dy).unwrap())
            .expect("find nearest neighbor")
            .into()
    }

    /// Obtains nearest neighbor and separation distance for a Histogram
    /// using lemma 1 from Elkan (2003) to avoid redundant distance
    /// calculations. Allowing us to efficiently assign each point to
    /// its initial centroid.
    fn create_centroids_tri_ineq(&self) -> Vec<Neighbor> {
        use indicatif::ParallelProgressIterator;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;

        // Initialization first half: d(c, c') for all centers c and c'
        // (this lets us use lemma 1 for massive speedups below)
        //
        // TODO: Extract into shared helper function (curently duped
        // here and in the main triangle accelerated clustering loop)
        let k = self.street().k();
        log::debug!("{:<32}", "precomputing centroid to centroid distances");
        let centroid_to_centroid_distances: Vec<Vec<f32>> = self
            .kmeans()
            .iter()
            // Get all combinations [(c1,c1), (c1,c2), ... (c_k, c_k)] into
            // a simple 1-D vector to allow for easily parallelizing the emd
            // calculations.
            // TLDR: effectively just itertools.array_combinations().
            .flat_map(|c| self.kmeans().iter().map(move |c_prime| (c, c_prime)))
            .collect::<Vec<_>>()
            .par_iter()
            .map(|(center1, center2)| self.emd(center1, center2)) // 1-D vector with length k^2
            .collect::<Vec<f32>>()
            .chunks(k) // Separate into k-length chunks so we can get it into a 2-D vector
            .map(|chunked| chunked.to_vec())
            .collect();

        log::debug!("{:<32}", "lemma 1 accelerated par_init of helpers");
        let nearest_neighbors: Vec<Neighbor> = self
            .points()
            .par_iter()
            // TODO: Add styling so that this matches all the other progress bars!
            .progress_count(self.points().len().try_into().unwrap())
            .map(|point| {
                // As of Jun 8, toggling this gives same results for shortdeck turn clustering.
                // So if no changes since then can assume that this is doing the correct thing for now.
                // TODO: ADD A TEST FOR THIS TOO!
                // Compute min distance d(x, c) efficiently by using
                // lemma 1 from Elkan (2003):
                // if d(b, c) >= 2d(x, b) then d(x, c) >= d(x, b)
                // Initially setting b as the 0-indexed centroid...
                let (index, initial_distance) = (0, self.emd(point, &(self.kmeans()[0])));
                // ... then continuing on for every other c
                let nearest_neighbor: Neighbor = self.kmeans().iter().enumerate().skip(1).fold(
                    (index, initial_distance),
                    |acc, x_enumerated| {
                        // center b index, d(x, b)
                        let (acc_center_index, acc_center_distance) = acc;
                        // center c index and histogram
                        let (next_center_index, next_center) = x_enumerated;

                        // Cheap lookup of precomputed d(b, c)
                        let distance_acc_centroid_to_next_centroid =
                            centroid_to_centroid_distances[acc_center_index][next_center_index];

                        // if d(b, c) >= 2d(x, b)...
                        if distance_acc_centroid_to_next_centroid >= 2.0 * acc_center_distance {
                            // ... then we definitely know d(x, c) >= d(x, b).
                            // So no need to do the distance calculation of d
                            // (x, c), we can just stick with the current
                            // center in our accumulator!
                            return acc;
                        }
                        // ... then it's not _necessarily_ true that d(x, c) >= d(x, b). So
                        // we have to actually do the distance calculation d(x, c) to find
                        // out whether or not this next centroid is in fact closer.
                        let next_center_distance = self.emd(point, next_center);
                        if next_center_distance >= acc_center_distance {
                            return acc; // this centroid was NOT closer when we checked
                        }
                        (next_center_index, next_center_distance) // this centroid WAS closer
                    },
                );
                nearest_neighbor
            })
            .collect();
        nearest_neighbors
    }

    /// reference to current street
    fn street(&self) -> Street {
        self.street
    }
    /// take outer triangular product of current learned kmeans
    /// Histograms, using whatever is stored as the future metric
    fn metric(&self) -> Metric {
        log::info!("{:<32}{:<32}", "calculating metric", self.street());
        let mut metric = BTreeMap::new();
        for (i, x) in self.kmeans.iter().enumerate() {
            for (j, y) in self.kmeans.iter().enumerate() {
                if i > j {
                    let ref a = self.abstraction(i);
                    let ref b = self.abstraction(j);
                    let index = Pair::from((a, b));
                    let distance = self.metric.emd(x, y) + self.metric.emd(y, x);
                    let distance = distance / 2.;
                    metric.insert(index, distance);
                }
            }
        }
        Metric::from(metric)
    }
    /// in AbsIterator order, get a mapping of
    /// Abstraction -> Histogram
    /// end-of-recurse call
    fn decomp(&self) -> Decomp {
        log::info!("{:<32}{:<32}", "calculating transitions", self.street());
        self.kmeans()
            .iter()
            .cloned()
            .enumerate()
            .map(|(k, centroid)| (self.abstraction(k), centroid))
            .collect::<BTreeMap<Abstraction, Histogram>>()
            .into()
    }
}

impl crate::save::disk::Disk for Layer {
    fn name() -> String {
        format!(
            "{:<16}{:<16}{:<16}",
            Lookup::name(),
            Decomp::name(),
            Metric::name()
        )
    }
    fn done(street: Street) -> bool {
        Lookup::done(street) && Decomp::done(street) && Metric::done(street)
    }
    fn save(&self) {
        self.metric().save();
        self.lookup().save();
        self.decomp().save();
    }
    fn grow(street: Street) -> Self {
        let layer = match street {
            Street::Rive => Self {
                street,
                kmeans: Vec::default(),
                points: Vec::default(),
                metric: Metric::default(),
            },
            _ => Self {
                street,
                kmeans: Vec::default(),
                points: Lookup::load(street.next()).projections(),
                metric: Metric::load(street.next()),
            },
        };
        layer.cluster()
    }
    fn load(_: Street) -> Self {
        unimplemented!()
    }
}
