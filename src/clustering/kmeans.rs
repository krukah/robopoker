use super::histogram::Histogram;
use crate::Energy;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterAlgorithm {
    KmeansOriginal = 0isize,

    /// Accelerated via Triangle Inequality math as per paper 'Elkan 2003'.
    KmeansElkan2003 = 1isize,
}

#[derive(Debug, Clone)]
pub struct ClusterArgs<'a> {
    /// Explicitly choose which clustering algorithm to use. Leave the optional
    /// empty to let cluster() choose.
    pub algorithm: ClusterAlgorithm,

    /// Center Histograms prior to performing any clustering / the start of the
    /// first training loop.
    ///
    /// The length of the resulting clusters will match, i.e. we look at this
    /// field's length to determine the 'k' for kmeans.
    /// TODO: Stop needing to pass ownership for this /
    /// use a different approach to update every iteration
    /// TODO: Determine whether this actually needs to be
    /// passed in in the first place...
    pub init_centers: Vec<Histogram>,

    /// Points to be clustered.
    pub points: &'a Vec<Histogram>,

    /// Number of training iterations to perform.
    pub iterations_t: usize,

    /// Used to tag logged messages. Solely for debugging purposes!
    pub label: String,

    /// Whether to compute the RMS of the resulting clusters in situations where
    /// doing so is not (effectively) "free", e.g. where we already have all the
    /// needed distances as a consequence of running the algorithm.
    ///
    /// Even if false RMS will still always be returned for other ClusterAlgorithm-s
    /// where it is (effecitvely) "free", e.g. KmeansOriginal.
    pub compute_rms: bool,
}

type Neighbor = (usize, f32);

/// "Carr[ied]... information" between k-means iterations for a specific point
/// in self.points. See Elkan 2003 for more details.
///
/// Used to accelerate k-means clustering via the paper's Triangle Inequality
/// (abrv. 'TriIneq' here) based optimized algorithm.
///
/// NOTE: Includes some additional fields besides _just_ the bounds. (E.g. a
/// field to help lookup the currently assigned centroid for the point).
///
/// TODO: Stop making this public / start hiding away some of the kmeans
/// logic so it's not directly in the trait
#[derive(Debug, Clone)]
struct TriIneqBounds {
    /// The index into self.kmeans for the currently assigned centroid "nearest
    /// neighbor" (i.e. c(x) in the paper) for this specifed point.
    assigned_centroid_idx: usize,
    /// Lower bounds on the distance from this point to each centroid c
    /// (l(x,c) in the paper).
    /// Is k in length, where k is the number of centroids in the k-means
    /// clustering. Each value inside the vector must correspond to the
    /// same-indexed **centroid** (not point!) in the Layer.
    lower_bounds: Vec<f32>,
    /// The upper bound on the distance from this point to its currently
    /// assinged centroid (u(x) in the paper).
    upper_bound: f32,
    /// Whether the upper_bound is out-of-date and needs a 'refresh'
    /// (r(x) from the paper).
    stale_upper_bound: bool,
}

pub trait Clusterable {
    /// TODO: consider updating this to use generics here if possible, + return a simple f32. E.g. some
    /// function <T1, T2> that does ((T1, T2, T2) -> f32).
    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy;
    /// TODO: remove this entirely and just have an implementation of this defined in terms
    /// of the distance function... if possible.
    fn nearest_neighbor(&self, clusters: &Vec<Histogram>, x: &Histogram) -> (usize, f32);
}

pub fn cluster<T: Clusterable + std::marker::Sync>(
    clusterable: &T,
    // TODO: update to avoid needing ownership, if possible
    cluster_args: ClusterArgs,
) -> (
    // Resulting clusters
    Vec<Histogram>,
    // RMS error at each iteration. Will be left empty unless either A.
    // compute_rms, or B. the RMSs are computed "for free" using the
    // specified ClusterAlgorithm (i.e. as a byproduct of performing the
    // clustering).
    Vec<f32>,
) {
    log::info!("{:<32}{:<32}", "initialize  kmeans", cluster_args.label);

    // Means the 'k' in kmeans is 0, so no work to do here.
    if cluster_args.init_centers.len() == 0 {
        log::debug!("Immediately returning empty values (since init_centers was empty / the 'k' in kmeans here is 0).");
        let empty_clusters = Vec::new();
        let empty_rms = Vec::new();
        return (empty_clusters, empty_rms);
    }
    let mut working_centers = cluster_args.init_centers.clone();
    let t = cluster_args.iterations_t;

    let mut all_rms: Vec<f32> = Vec::default();
    match cluster_args.algorithm {
        ClusterAlgorithm::KmeansOriginal => {
            log::info!(
                "{:<32}{:<32}",
                "clustering kmeans (unoptimized)",
                cluster_args.label
            );
            let progress = crate::progress(t);
            for _ in 0..t {
                let (next_centers, rms) =
                    compute_next_kmeans(clusterable, &cluster_args, &working_centers);
                log::debug!("{:<32}{:<32}", "abstraction cluster RMS error", rms);
                all_rms.push(rms);

                working_centers = next_centers;
                progress.inc(1);
            }
            progress.finish();
            println!();
        }
        ClusterAlgorithm::KmeansElkan2003 => {
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
            let mut ti_helpers: Vec<TriIneqBounds> =
                create_centroids_tri_ineq(clusterable, &cluster_args)
                    .iter()
                    // TODO: Double check we're not repeating the 'pick initial centers' work here twice.
                    // (e.g. if we already did that during the init() above)
                    .map(|nearest_neighbor| TriIneqBounds {
                        // "c(x)"'s index in init_centers
                        assigned_centroid_idx: nearest_neighbor.0,
                        // "l(x,c)"
                        // "Set the lower bound l(x,c) = 0 for each point x and center c"
                        lower_bounds: vec![0.0; cluster_args.init_centers.len()],
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
                "clustering kmeans (Elkan 2003)",
                cluster_args.label
            );
            // As per the paper:
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
            let mp = MultiProgress::new();
            let progress = mp.add(crate::progress(t));
            // Ensures that the progress bar actually refreshes smoothly (as opposed
            // to e.g. hanging out at 4%, then jumping all the way to 40%)
            // (Could probably be )
            progress.enable_steady_tick(Duration::from_millis(500));

            for i in 0..t {
                progress.inc(1);

                log::debug!("{:<32}{:<32}", "Performing training iteration # ", i);
                let result = compute_next_kmeans_tri_ineq(
                    clusterable,
                    &cluster_args,
                    &working_centers,
                    &ti_helpers,
                    Some(&mp),
                );

                if let Some(rms) = result.rms {
                    log::debug!("{:<32}{:<32}", "abstraction cluster RMS error", rms);
                    all_rms.push(rms);
                }

                working_centers = result.centers;
                ti_helpers = result.helpers;
            }
        }
    }
    (working_centers, all_rms)
}

#[cfg(feature = "native")]
/// calculates the next step of the kmeans iteration by
/// determining K * N optimal transport calculations and
/// taking the nearest neighbor
fn compute_next_kmeans<T: Clusterable + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs,
    centers_start: &Vec<Histogram>,
) -> (Vec<Histogram>, f32) {
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    let k = cluster_args.init_centers.len();
    let mut loss = 0f32;
    let mut centers_end = vec![Histogram::default(); k];
    // assign points to nearest neighbors
    for (point, (neighbor, distance)) in cluster_args
        .points
        .par_iter()
        .map(|h| (h, clusterable.nearest_neighbor(centers_start, h)))
        .collect::<Vec<_>>()
        .into_iter()
    {
        loss = loss + distance * distance;
        centers_end
            .get_mut(neighbor)
            .expect("index from neighbor calculation")
            .absorb(point);
    }
    let rms = (loss / cluster_args.points.len() as f32).sqrt();
    (centers_end, rms)
}

/// Helper function to replace a 'spinner' ProgressBar inside the specificed MultiProgress
/// with a new one carrying a different message.
///
/// This is useful for starting/stopping spinners repeatedly to help provide visual signal that we are "doing stuff"
/// in places where it's otherwise impractical to show the exact progress (e.g. when A. we need to
/// use a MultiProgress becasue we already have at least one bar going, and
/// also at the same time B. we're using Rayon / parallelization and so cannot manually incrememnt
/// things ourselves).
fn replace_multiprogress_spinner(
    multi_progress: &Option<&MultiProgress>,
    finished_spinner: ProgressBar,
    message: String,
) -> ProgressBar {
    let mut running_spinner = finished_spinner;
    if let Some(mp) = multi_progress {
        mp.remove(&running_spinner);
        running_spinner = mp.add(ProgressBar::new_spinner());
    }

    running_spinner.set_message(message);
    running_spinner.enable_steady_tick(Duration::from_millis(10));
    running_spinner
}

// Simple helper for specifically the Elkan 2003 Triangle-Inequality accelerated version
// of Kmeans to make it easier to pass around the result.
//
// See below for more details.
#[derive(Debug)]
struct ElkanIterationResult {
    // K centroids
    centers: Vec<Histogram>,
    // Updated Triangle Inequality Helpers after each iteration
    helpers: Vec<TriIneqBounds>,
    // RMS error iff compute_rms is enabled
    rms: Option<f32>,
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
fn compute_next_kmeans_tri_ineq<T: Clusterable + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs,
    // The centers at the start of *this training iteration*.
    // (WARNING: do not confuse with cluster_args.init_centers!)
    centers_start: &Vec<Histogram>,
    ti_helpers: &[TriIneqBounds],
    multi_progress: Option<&MultiProgress>,
) -> ElkanIterationResult {
    // TODO: panic if the length of ti_helpers doesn't match the length of
    // self.points

    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    let mut spinner: ProgressBar = ProgressBar::new_spinner();

    let k = cluster_args.init_centers.len();

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
    let centroid_to_centroid_distances: Vec<Vec<f32>> = centers_start
        .iter()
        // Get all combinations [(c1,c1), (c1,c2), ... (c_k, c_k)] into
        // a simple 1-D vector to allow for easily parallelizing the emd
        // calculations.
        // TLDR: effectively just itertools.array_combinations().
        .flat_map(|c| centers_start.iter().map(move |c_prime| (c, c_prime)))
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(center1, center2)| clusterable.distance(center1, center2)) // 1-D vector with length k^2
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

    let mut step_3_working_points: HashMap<usize, (&Histogram, TriIneqBounds)> = cluster_args
        .points
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
    spinner = replace_multiprogress_spinner(&multi_progress, spinner, "(Elkan Step 3)".to_string());
    use rayon::prelude::*;
    for (center_c_idx, center_c) in centers_start.iter().enumerate() {
        step_3_working_points
            .par_iter_mut()
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
                    let dist =
                        clusterable.distance(point_h, &centers_start[helper.assigned_centroid_idx]);
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
                    let dist_to_center_c = clusterable.distance(point_h, center_c);
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
    spinner.finish();

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

    spinner = replace_multiprogress_spinner(&multi_progress, spinner, "(Elkan Step 4)".to_string());

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
    let points_assigned_per_center: Vec<Vec<&Histogram>> = centers_start
        .iter()
        .enumerate()
        .map(|(center_c_idx, _center_c)| {
            step_4_helpers
                .iter()
                .enumerate()
                .filter(|(_point_i, helper)| helper.assigned_centroid_idx == center_c_idx)
                .map(|(point_i, _)| &cluster_args.points[point_i])
                .collect()
        })
        .collect();

    if cluster_args.compute_rms {
        log::debug!(
            "Performing {} otherwise-unnecessary emd computations to
             calculate RMS error. This may slow down step 4!",
            cluster_args.points.len()
        );
    }
    let mut loss = 0f32;
    let mut rms_calculation_seconds = 0;

    let mut new_centroids: Vec<Histogram> = vec![];

    // Note: we could optionally parallelize this. In practice though, so long
    // as `perform_extra_loss_calculations` is false this is so fast that it's
    // not worth doing.
    for (centroid_i, points) in points_assigned_per_center.iter().enumerate() {
        if points.is_empty() {
            log::error!("No points assigned to current centroid. This is currently an edge case we are unable to resolve; for more details see https://github.com/krukah/robopoker/issues/34#issuecomment-2860641178");
            todo!(
                "Figure out what to do here for the center - no points assigned to it currently!!"
            );
        }
        let mut mean_of_assigned_points = points[0].clone();

        for point in points.iter().skip(1) {
            mean_of_assigned_points.absorb(point);
        }
        let next_centroid = mean_of_assigned_points;

        if cluster_args.compute_rms {
            // NOTE: Calculating the error with the OLD center (to ensure
            // that this is consistent with the unaccelerated algorithm).
            let old_centroid = &(centers_start[centroid_i]);
            // As mentioned above, this can be expensive; we add extra tracking
            // here to allow the user to more easily determine if it's worth
            // disabling or not.
            let now = SystemTime::now();
            for point in points.iter() {
                let distance_point_to_prior_centroid = clusterable.distance(&old_centroid, &point);
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

    let mut optional_rms: Option<f32> = None;
    if cluster_args.compute_rms {
        spinner.finish();
        spinner = replace_multiprogress_spinner(&multi_progress, spinner, "(Performing optional 'non-free' RMS calculations. Consider disabling them if doing performance testing!)".to_string());

        let rms = (loss / cluster_args.points.len() as f32).sqrt();
        optional_rms = Some(rms);

        // Arbitrarily setting a threshold for when it 'affects' performance;
        // anything under is clearly so small that the reminder message isn't
        // needed. (Arguably we could set this much higher too.)
        if rms_calculation_seconds > 2 {
            log::warn!(
                "Computing RMS values required {} seconds
                of extra work ({} distance computations).",
                rms_calculation_seconds,
                cluster_args.points.len()
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
        .zip(centers_start)
        .map(|(old_center, new_center)| clusterable.distance(old_center, new_center))
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

    spinner.finish();
    if let Some(mp) = multi_progress {
        mp.remove(&spinner)
    }

    // Form paper "[Compute] the new location of each cluster center",
    // i.e. Step 7:
    // "7. Replace each center c by m(c)"
    log::debug!("{:<32}", " - Elkan Step 7");

    ElkanIterationResult {
        centers: new_centroids,
        helpers: step_6_helpers,
        rms: optional_rms,
    }
}

/// Obtains nearest neighbor and separation distance for a Histogram
/// using lemma 1 from Elkan (2003) to avoid redundant distance
/// calculations. Allowing us to efficiently assign each point to
/// its initial centroid.
fn create_centroids_tri_ineq<T: Clusterable + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs,
) -> Vec<Neighbor> {
    use crate::PROGRESS_STYLE;
    use indicatif::ParallelProgressIterator;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;

    // Initialization first half: d(c, c') for all centers c and c'
    // (this lets us use lemma 1 for massive speedups below)
    //
    // TODO: Extract into shared helper function (curently duped
    // here and in the main triangle accelerated clustering loop)
    let k = cluster_args.init_centers.len();
    log::debug!("{:<32}", "precomputing centroid to centroid distances");
    let centroid_to_centroid_distances: Vec<Vec<f32>> = cluster_args
        .init_centers
        .iter()
        // Get all combinations [(c1,c1), (c1,c2), ... (c_k, c_k)] into
        // a simple 1-D vector to allow for easily parallelizing the emd
        // calculations.
        // TLDR: effectively just itertools.array_combinations().
        .flat_map(|c| {
            cluster_args
                .init_centers
                .iter()
                .map(move |c_prime| (c, c_prime))
        })
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(center1, center2)| clusterable.distance(center1, center2)) // 1-D vector with length k^2
        .collect::<Vec<f32>>()
        .chunks(k) // Separate into k-length chunks so we can get it into a 2-D vector
        .map(|chunked| chunked.to_vec())
        .collect();

    log::debug!("{:<32}", "lemma 1 accelerated par_init of helpers");
    let style = indicatif::ProgressStyle::with_template(PROGRESS_STYLE).unwrap();
    log::debug!("{}", PROGRESS_STYLE);

    let nearest_neighbors: Vec<Neighbor> = cluster_args
        .points
        .par_iter()
        .progress_with_style(style)
        .map(|point| {
            // Compute min distance d(x, c) efficiently by using
            // lemma 1 from Elkan (2003):
            // if d(b, c) >= 2d(x, b) then d(x, c) >= d(x, b)
            // Initially setting b as the 0-indexed centroid...
            let (index, initial_distance) = (
                0,
                clusterable.distance(point, &(cluster_args.init_centers[0])),
            );
            // ... then continuing on for every other c
            let nearest_neighbor: Neighbor =
                cluster_args.init_centers.iter().enumerate().skip(1).fold(
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
                        let next_center_distance = clusterable.distance(point, next_center);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arbitrary;
    use crate::Energy;

    #[derive(Debug)]
    struct MockClusterable {}
    impl Clusterable for MockClusterable {
        // Smple Euclidean-like distance for testing
        fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
            let dist1 = h1.distribution();
            let dist2 = h2.distribution();

            // Simple distance calculation based on distribution differences
            let mut sum = 0.0;
            let all_keys: std::collections::HashSet<_> =
                dist1.iter().chain(dist2.iter()).map(|(k, _)| k).collect();
            for &key in &all_keys {
                let p1 = h1.density(key);
                let p2 = h2.density(key);
                sum += (p1 - p2).powi(2);
            }
            sum.sqrt()
        }

        fn nearest_neighbor(&self, clusters: &Vec<Histogram>, x: &Histogram) -> (usize, f32) {
            clusters
                .iter()
                .enumerate()
                .map(|(i, cluster)| (i, self.distance(x, cluster)))
                .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal))
                .expect("find nearest neighbor")
        }
    }

    fn create_seeded_histograms(i: i32) -> Vec<Histogram> {
        (0..i).map(|_| Histogram::random()).collect()
    }

    #[test]
    fn test_kmeans_elkan_rms_converges() {
        let points: Vec<Histogram> = create_seeded_histograms(400);
        let init_centers: Vec<Histogram> = create_seeded_histograms(2);

        let clusterable = MockClusterable {};
        let cluster_args = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansElkan2003,
            init_centers,
            points: &points,
            iterations_t: 6,
            label: "test_elkan_converges".to_string(),
            compute_rms: true,
        };

        let (result, all_rms) = cluster(&clusterable, cluster_args);
        for w in all_rms.windows(2) {
            println!("{} {}", w[0], w[1]);
        }

        // "Safety" checks to make sure things are overall behaving as expected
        assert_eq!(result.len(), 2);
        assert_eq!(all_rms.len(), 6);
        assert!(
            (all_rms[0] - all_rms[1]).abs() > 0.01,
            "RMS was already converged too much for a fair test at the start (goes from {} to {})",
            all_rms[0],
            all_rms[1]
        );

        // Actual asserts the test is targeting ("did the RMS converge near the end")
        for w in all_rms.into_iter().skip(3).collect::<Vec<_>>().windows(2) {
            let prior_rms = w[0];
            let next_rms = w[1];
            println!("{} {}", prior_rms, next_rms);

            assert!(
                (prior_rms - next_rms).abs() <= 0.0005,
                "RMS is still decreasing _too much_ / did not converge enough (goes from {} to {})",
                prior_rms,
                next_rms
            );
        }
    }

    #[test]
    fn test_kmeans_elkan_rms_decreases() {
        let points: Vec<Histogram> = create_seeded_histograms(400);
        let init_centers: Vec<Histogram> = create_seeded_histograms(5);

        let clusterable = MockClusterable {};
        let cluster_args = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansElkan2003,
            init_centers,
            points: &points,

            // Don't set too high; the values stop decreasing as much in normal operation once it starts converging.
            iterations_t: 4,

            label: "test_elkan_decreases".to_string(),
            compute_rms: true,
        };

        let (result, all_rms) = cluster(&clusterable, cluster_args);
        assert_eq!(result.len(), 5);
        assert_eq!(all_rms.len(), 4);

        for w in all_rms.windows(2) {
            let prior_rms = w[0];
            let next_rms = w[1];
            println!("{} {}", prior_rms, next_rms);

            assert!(
                next_rms < prior_rms,
                "RMS was not monotonically decreasing (goes from {} to {})",
                prior_rms,
                next_rms
            );
            assert!(
                (prior_rms - next_rms).abs() > 0.0001,
                "RMS did not decrease *enough* during at least one iteration (goes from {} to {})",
                prior_rms,
                next_rms
            );
        }
    }

    #[test]
    fn test_kmeans_original_rms_decreases() {
        let points: Vec<Histogram> = create_seeded_histograms(400);
        let init_centers: Vec<Histogram> = create_seeded_histograms(5);
        let clusterable = MockClusterable {};
        let cluster_args = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansOriginal,
            init_centers,
            points: &points,

            // Don't set too high; the values stop decreasing as much in normal operation once it starts converging
            iterations_t: 4,

            label: "test_original".to_string(),
            compute_rms: true,
        };

        let (result, all_rms) = cluster(&clusterable, cluster_args);
        assert_eq!(result.len(), 5);
        assert_eq!(all_rms.len(), 4);

        for w in all_rms.windows(2) {
            let prior_rms = w[0];
            let next_rms = w[1];
            assert!(
                next_rms < prior_rms,
                "RMS was not monotonially decreasing (goes from {} to {})",
                prior_rms,
                next_rms
            );
            assert!(
                (prior_rms - next_rms).abs() > 0.0001,
                "RMS did not decrease *enough* during at least one iteration (goes from {} to {})",
                prior_rms,
                next_rms
            );
            println!("{} {}", prior_rms, next_rms);
        }
    }

    /// As per the research paper:
    /// "After each iteration, [Elkan's algorithm] produces the same set of center locations as the standard k-means method."
    /// Therefore, the RMS we compute at every single iteration should be (nearly) identical.
    #[test]
    fn test_kmeans_elkan_original_rms_matches() {
        let points_elkan: Vec<Histogram> = create_seeded_histograms(400);
        let points_original: Vec<Histogram> = points_elkan.clone();
        let init_centers_elkan: Vec<Histogram> = create_seeded_histograms(3);
        let init_centers_original: Vec<Histogram> = init_centers_elkan.clone();
        let clusterable = MockClusterable {};
        let cluster_args_elkan = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansElkan2003,
            init_centers: init_centers_elkan,
            points: &points_elkan,
            iterations_t: 4,
            label: "test_elkan".to_string(),
            compute_rms: true,
        };
        let cluster_args_original = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansOriginal,
            init_centers: init_centers_original,
            points: &points_original,
            label: "test_original".to_string(),
            ..cluster_args_elkan
        };

        let (_, all_rms_elkan) = cluster(&clusterable, cluster_args_elkan);
        let (_, all_rms_original) = cluster(&clusterable, cluster_args_original);
        assert_eq!(all_rms_elkan.len(), 4);
        assert_eq!(all_rms_original.len(), 4);

        for (elkan_rms, original_rms) in all_rms_elkan.iter().zip(all_rms_original) {
            println!("elkan: {}, original: {}", elkan_rms, original_rms);
            assert!(
                (elkan_rms - original_rms).abs() < 0.00001,
                "RMS-es (elkan: {}, original: {}) should approximately match at each step",
                elkan_rms,
                original_rms
            )
        }
    }
}
