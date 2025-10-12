use super::*;
use crate::Energy;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use std::collections::*;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterAlgorithm {
    KmeansOriginal = 0isize,

    /// Accelerated via Triangle Inequality math as per paper 'Elkan 2003'.
    ///
    /// Guaranteed to return approximately-identical centers to those computed
    /// by the KmeansOriginal algorithm (when clustering the same set inputs).
    KmeansElkan2003 = 1isize,
}

#[derive(Debug, Clone)]
pub struct ClusterArgs<'a, D> {
    /// Explicitly choose which clustering algorithm to use.
    ///
    /// Note: we suggest using KmeansElkan2003 over KMeansOriginal in most
    /// cases. Given the same inputs they return approximately-identical
    /// centers at each iteration, and the former is usually _significantly_
    /// faster.
    pub algorithm: ClusterAlgorithm,

    /// Center Histograms prior to performing any clustering / the start of
    /// the first training loop.
    ///
    /// The length of the resulting clusters will match, i.e. we look at this
    /// field's length to determine the 'k' for kmeans.
    pub init_centers: &'a Vec<D>,

    /// Points to be clustered.
    pub points: &'a Vec<D>,

    /// Number of training iterations to perform.
    pub iterations_t: usize,

    /// Used to tag logged messages. Solely for debugging purposes!
    pub label: String,

    /// Whether to compute the RMS of the resulting clusters in situations
    /// where doing so is not (effectively) "free", e.g. where we already
    /// have all the needed distances as a consequence of running the
    /// algorithm.
    ///
    /// Even if false RMS will still always be returned for other
    /// ClusterAlgorithm-s where it is (effecitvely) "free", e.g.
    /// KmeansOriginal.
    pub compute_rms: bool,
}

pub trait KMeans: Sync {
    type P: Absorb + Send + Sync + Clone;

    fn t(&self) -> usize;
    fn k(&self) -> usize;
    fn n(&self) -> usize;

    fn points(&self) -> &Vec<Self::P>;
    fn centers(&self) -> &Vec<Self::P>;
    fn metadata(&self) -> &Vec<Bounds>;

    fn assert(&self) {
        assert!(self.n() == self.metadata().len());
        assert!(self.n() == self.points().len());
        assert!(self.k() == self.centers().len());
    }

    fn distance(&self, h1: &Self::P, h2: &Self::P) -> Energy;

    /// Compute the nearest neighbor in O(k) * MetricCost
    fn neighbor(&self, x: &Self::P) -> (usize, f32) {
        self.centers()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, self.distance(c, x)))
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
            .unwrap()
    }

    /// Compute d(c, c') for all centers c and c'
    fn pairwise(&self) -> Vec<Vec<f32>> {
        self.centers()
            .iter()
            .flat_map(|c1| self.centers().iter().map(move |c2| self.distance(c1, c2)))
            .collect::<Vec<_>>()
            .chunks(self.n())
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Compute s(c) = (1/2) min_{c'!=c} d(c, c')
    fn midpoints(&self) -> Vec<f32> {
        self.pairwise()
            .iter()
            .enumerate()
            .map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, &d)| d)
                    .reduce(f32::min)
                    .map(|d| d * 0.5)
                    .unwrap_or(0.0)
            })
            .collect()
    }

    /// Identify points where u(x) <= s(c(x))
    fn exclusions(&self) -> HashSet<usize> {
        let midpoints = self.midpoints();
        self.metadata()
            .iter()
            .enumerate()
            .filter(|(_, bs)| bs.upper <= *midpoints.get(bs.j).unwrap())
            .map(|(x, _)| x)
            .collect()
    }

    /// Identify points where u(x) <= s(c(x)) and c(x) != c(y)
    fn inclusions(&self) -> HashMap<usize, (&Self::P, &Bounds)> {
        let exclusions = self.exclusions();
        self.points()
            .iter()
            .enumerate()
            .filter(|(i, _)| !exclusions.contains(i))
            .map(|(i, p)| (i, (p, self.metadata().get(i).unwrap())))
            .collect()
    }

    // ====================================================================
    // Elkan 2003 Step 4: Merge and recompute centers
    // ====================================================================

    /// Merge updated bounds back with original
    fn elkan_step4_merge_helpers<'a>(
        &self,
        per_point_metadata: &'a [Bounds],
        step_3_working_points: &'a HashMap<usize, (&Self::P, Bounds)>,
    ) -> Vec<&'a Bounds> {
        per_point_metadata
            .iter()
            .enumerate()
            .map(|(point_i, original_helper)| {
                if step_3_working_points.contains_key(&point_i) {
                    &(step_3_working_points[&point_i].1)
                } else {
                    original_helper
                }
            })
            .collect()
    }

    /// Group points by assigned centroid
    fn elkan_step4_points_per_center(
        &self,
        centers_start: &[Self::P],
        step_4_helpers: &[&Bounds],
        points: &[Self::P],
    ) -> Vec<Vec<&Self::P>> {
        centers_start
            .iter()
            .enumerate()
            .map(|(center_c_idx, _center_c)| {
                step_4_helpers
                    .iter()
                    .enumerate()
                    .filter(|(_point_i, helper)| helper.j == center_c_idx)
                    .map(|(point_i, _)| &points[point_i])
                    .collect()
            })
            .collect()
    }

    /// Compute new centroids from assigned points
    fn elkan_step4_compute_centroids(
        &self,
        points_assigned_per_center: &[Vec<&Self::P>],
    ) -> Vec<Self::P> {
        let mut new_centroids = vec![];
        for points in points_assigned_per_center.iter() {
            if points.is_empty() {
                log::error!("No points assigned to centroid");
                todo!("Handle empty centroid assignment");
            }
            let mut mean = points[0].clone();
            for point in points.iter().skip(1) {
                mean.absorb(point);
            }
            new_centroids.push(mean);
        }
        new_centroids
    }

    // ====================================================================
    // Elkan 2003 Step 5 & 6: Update bounds
    // ====================================================================

    /// Compute d(c, m(c)) movements
    fn elkan_compute_movements(
        &self,
        new_centroids: &[Self::P],
        centers_start: &[Self::P],
    ) -> Vec<f32> {
        use rayon::iter::IndexedParallelIterator;
        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        new_centroids
            .par_iter()
            .zip(centers_start.par_iter())
            .map(|(new_center, old_center)| self.distance(old_center, new_center))
            .collect()
    }

    /// Step 5: Update lower bounds
    fn elkan_step5_lower_bounds(
        &self,
        step_4_helpers: Vec<&Bounds>,
        new_centroid_movements: &[f32],
    ) -> Vec<Bounds> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;

        step_4_helpers
            .into_par_iter()
            .cloned()
            .map(|mut helper| {
                for (lower_bound, &centroid_movement) in
                    helper.lower.iter_mut().zip(new_centroid_movements)
                {
                    *lower_bound = (*lower_bound - centroid_movement).max(0.0);
                }
                helper
            })
            .collect()
    }

    /// Step 6: Update upper bounds
    fn elkan_step6_upper_bounds(
        &self,
        mut step_5_helpers: Vec<Bounds>,
        new_centroid_movements: &[f32],
    ) -> Vec<Bounds> {
        for helper in &mut step_5_helpers {
            let dist_center_and_new_center = &new_centroid_movements[helper.j];
            helper.upper += dist_center_and_new_center;
            helper.stale = true;
        }
        step_5_helpers
    }
}

type Neighbor = (usize, f32);

#[cfg(feature = "native")]
/// Calculates the next step of the kmeans iteration by determining K * N
/// optimal transport calculations and taking the nearest neighbor.
fn compute_next_kmeans<T: KMeans + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs<T::P>,
    centers_start: &Vec<T::P>,
) -> (Vec<T::P>, f32) {
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    let k = cluster_args.init_centers.len();
    let mut loss = 0f32;
    let mut centers_end = vec![T::P::default(); k];
    // assign points to nearest neighbors
    for (point, (neighbor, distance)) in clusterable
        .points()
        .iter()
        .map(|p| (p, clusterable.neighbor(p)))
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

#[cfg(feature = "native")]
/// Elkan 2003 Triangle Inequality accelerated version of kmeans.
///
/// TODO: Refactor this imperative function to use the new functional trait methods:
/// - elkan_step1_metadata, elkan_step2_filter, elkan_step3_update_bounds, etc.
/// This will reduce ~400 lines to ~50 lines of composable functional code.
/// See the trait method implementations for the refactored logic.
///
/// Elkan 2003 Triangle Inequality accelerated version of kmeans.
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
fn compute_next_kmeans_elkan2003<T: KMeans + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs<T::P>,
    // The centers at the start of *this training iteration*.
    // (WARNING: do not confuse with cluster_args.init_centers!)
    cs: &Vec<T::P>,
    metadata: &[Bounds],
    multi_progress: Option<&MultiProgress>,
) -> ElkanIterationResult<T::P> {
    // Both by definition should be length 'N'.
    assert_eq!(metadata.len(), cluster_args.points.len());

    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    let mut spinner: ProgressBar = ProgressBar::new_spinner();

    // Step 1 (first half): d(c, c') for all centers c and c'
    let centroid_to_centroid_distances: Vec<Vec<f32>> =
        todo!("pairwise(clusterable, centers_start);;");
    // Step 1 (second half): s(c) = (1/2) min_{c'!=c} d(c, c')
    // (i.e. the closest midpoint to another centroid besides itself)
    let midpointz: Vec<f32> = todo!("midpoints");
    let pairwises: Vec<Vec<f32>> = todo!("pairwise(clusterable, centers_start);;");
    let exclusions = metadata
        .iter()
        .enumerate()
        .filter(|(_, bs)| bs.upper <= midpointz[bs.j])
        .map(|(x, _)| x)
        .collect::<HashSet<usize>>();
    let mut inclusions = self
        .points()
        .iter()
        .enumerate()
        .filter(|(i, _)| !exclusions.contains(i))
        .map(|(i, p)| (i, (p, metadata[i].clone())))
        .collect::<HashMap<usize, _>>();
    use rayon::prelude::*;
    for (j, c) in cs.iter().enumerate() {
        inclusions.par_iter_mut().for_each(|(_, (p, bounds))| {
            // STEP 3 FILTERING: Apply all three filter conditions with early exits
            // STEP 3.i: Skip if c == c(x) (point already assigned to this centroid)
            // STEP 3.ii: Skip if u(x) <= l(x, c) (upper bound not greater than lower bound)
            // STEP 3.iii: Skip if u(x) <= (1/2) * d(c(x), c)
            // (i.e. upper bound not greater than half centroid distance)
            if j == bounds.j
                || bounds.upper <= bounds.lower[j]
                || bounds.upper <= 0.5 * pairwises[bounds.j][j]
            {
                return;
            }
            // STEP 3.a: "If r(x) then compute d(x, c(x)) and assign r(x) = false.
            //           Otherwise, d(x, c(x)) = u(x)."
            let upper = if bounds.stale {
                let max = clusterable.distance(p, &cs[bounds.j]); //
                bounds.lower[bounds.j] = max; // Update l(x, c(x)) in-place
                bounds.upper = max; // Update u(x) in-place
                bounds.stale = false; // clear r (x) in-place
                max
            } else {
                bounds.upper
            };
            // Step 3.b:
            //  If d(x, c(x)) > l(x,c)
            //  or d(x, c(x)) > (1/2) d(c(x), c)
            // then:
            //  Compute d(x,c)
            //  If d(x,c) < d(x, c(x)) then assign c(x) = c
            if upper > bounds.lower[j] || upper > 0.5 * pairwises[bounds.j][j] {
                //  ... "Compute d(x,c)"
                let radius = clusterable.distance(p, c);
                // (As discussed above: "each time d(x, c) is calculated ...")
                bounds.lower[j] = radius; // update l(x,c) in place
                                          // ... If d(x,c) < d(x, c(x)) then assign c(x) = c
                if radius < upper {
                    bounds.j = j; // Reassign c(x) = c in-place
                    bounds.upper = radius // update u (x) in place
                }
            }
        });
    }

    log::debug!("{:<32}", " - Elkan 2003 Step 4");
    // Merge the updated helper values back with the original vector we got
    // at the start of the function (which has entries for *all* points, not
    // just the ones bieng updated in step 3).
    let step_4_helpers: Vec<&Bounds> = metadata
        .iter()
        .enumerate()
        .map(|(point_i, original_helper)| {
            if inclusions.contains_key(&point_i) {
                &(inclusions[&point_i].1)
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
    let assignments = cs
        .iter()
        .enumerate()
        .map(|(c, _)| {
            step_4_helpers
                .iter()
                .enumerate()
                .filter(|(_, bs)| bs.j == c)
                .map(|(i, _)| &cluster_args.points.get(i).expect("n bounds"))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
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
    for (j, points) in assignments.into_iter().enumerate() {
        if points.is_empty() {
            panic!();
        }
        let mut mean = points[0].clone();
        for point in points.iter().skip(1) {
            mean.absorb(point);
        }
        if cluster_args.compute_rms {
            let old_centroid = &(cs[j]);
            for point in points {
                loss += clusterable.distance(old_centroid, point).powi(2);
            }
        }
        new_centroids.push(mean);
    }

    let mut optional_rms: Option<f32> = None;
    if cluster_args.compute_rms {
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

    log::debug!("{:<32}", " - Elkan 2003 Step 5");
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
    // Note: assumes that the distance is equivalent in either direction.
    // (If not, would need to e.g. compute a Vec<(f32, f32)> instead.)
    let new_centroid_movements: Vec<f32> = new_centroids
        .par_iter()
        .zip(cs)
        .map(|(old_center, new_center)| clusterable.distance(old_center, new_center))
        .collect();

    let step_5_helpers: Vec<Bounds> = step_4_helpers
        .into_par_iter()
        .cloned()
        .map(|mut helper| {
            // Update lower_bounds in-place (sequential within each helper)
            // Note: looks up d(c, m(c)) from when we calculated it outside
            // the loop above.
            for (lower_bound, &centroid_movement) in
                helper.lower.iter_mut().zip(&new_centroid_movements)
            {
                *lower_bound = (*lower_bound - centroid_movement).max(0.0);
            }
            helper
        })
        .collect();

    log::debug!("{:<32}", " - Elkan 2003 Step 6");
    // Step 6: Update upper bounds. From paper: """
    // 6. For each point x, assign
    //    u(x) = u(x) + d(m(c(x)), c(x))
    //    r(x) = true
    // """
    // TODO: consider refactoring - we probably can get away with continuing
    // to borrow here? And/or do using a .map() inside in a .par_iter() etc?
    let mut step_6_helpers: Vec<Bounds> = step_5_helpers;
    for helper in &mut step_6_helpers {
        // u(x) = u(x) + d(m(c(x)), c(x))
        // Note: assumes that d(m(c(x)), c(x)) = d(c(x), m(c(x))).
        let dist_center_and_new_center = &new_centroid_movements[helper.j];
        helper.upper += dist_center_and_new_center;
        // r(x) = true
        helper.stale = true;
    }

    spinner.finish_and_clear();
    if let Some(mp) = multi_progress {
        mp.remove(&spinner)
    }

    // Still need to handle Step 7. i.e.
    //
    //  "7. Replace each center c by m(c)"
    //
    // but that all gets taken care of by the caller so no need to worry about
    // it inside here.
    log::debug!("{:<32}", " - Elkan 2003 Step 7");
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
fn create_centroids_tri_ineq<T: KMeans + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs<T::P>,
) -> Vec<Neighbor> {
    use crate::PROGRESS_STYLE;
    use indicatif::ParallelProgressIterator;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;

    // Initialization first half: d(c, c') for all centers c and c'
    // (this lets us use lemma 1 for massive speedups below)
    // let k = cluster_args.init_centers.len();
    log::debug!("{:<32}", "precomputing centroid to centroid distances");
    let centroid_to_centroid_distances: Vec<Vec<f32>> =
        pairwise(clusterable, &cluster_args.init_centers);

    log::debug!("{:<32}", "lemma 1 accelerated par_init of helpers");
    let style = indicatif::ProgressStyle::with_template(PROGRESS_STYLE).unwrap();

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

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
fn replace_multiprogress_spinner(
    multi_progress: &Option<&MultiProgress>,
    spinner: ProgressBar,
    message: String,
) -> ProgressBar {
    if !spinner.is_finished() {
        spinner.finish_and_clear();
    }
    let mut running_spinner = spinner;
    if let Some(mp) = multi_progress {
        mp.remove(&running_spinner);
        running_spinner = mp.add(ProgressBar::new_spinner());
    }

    running_spinner.set_message(message);
    running_spinner.enable_steady_tick(Duration::from_millis(10));
    running_spinner
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arbitrary;
    use crate::Energy;

    #[derive(Debug)]
    struct MockClusterable {}

    fn create_seeded_histograms(i: i32) -> Vec<Histogram> {
        (0..i).map(|_| Histogram::random()).collect()
    }

    #[test]
    fn test_kmeans_elkan2003_rms_converges() {
        let points: Vec<Histogram> = create_seeded_histograms(400);
        let init_centers: Vec<Histogram> = create_seeded_histograms(2);

        let clusterable = MockClusterable {};
        let cluster_args = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansElkan2003,
            init_centers: &init_centers,
            points: &points,
            iterations_t: 6,
            label: "test_elkan2003_converges".to_string(),
            compute_rms: true,
        };

        let (result, all_rms) = cluster(&clusterable, &cluster_args);
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
    fn test_kmeans_elkan2003_rms_decreases() {
        let points: Vec<Histogram> = create_seeded_histograms(500);
        let init_centers: Vec<Histogram> = create_seeded_histograms(5);

        let clusterable = MockClusterable {};
        let cluster_args = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansElkan2003,
            init_centers: &init_centers,
            points: &points,

            // Don't set too high; the values stop decreasing as much in normal operation once it starts converging.
            iterations_t: 4,

            label: "test_elkan2003_decreases".to_string(),
            compute_rms: true,
        };

        let (result, all_rms) = cluster(&clusterable, &cluster_args);
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
            init_centers: &init_centers,
            points: &points,

            // Don't set too high; the values stop decreasing as much in normal operation once it starts converging
            iterations_t: 4,

            label: "test_original".to_string(),
            compute_rms: true,
        };

        let (result, all_rms) = cluster(&clusterable, &cluster_args);
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
    fn test_kmeans_elkan2003_original_rms_matches() {
        let points_elkan: Vec<Histogram> = create_seeded_histograms(500);
        let points_original: Vec<Histogram> = points_elkan.clone();
        let init_centers_elkan: Vec<Histogram> = create_seeded_histograms(3);
        let init_centers_original: Vec<Histogram> = init_centers_elkan.clone();
        let clusterable = MockClusterable {};
        let cluster_args_elkan = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansElkan2003,
            init_centers: &init_centers_elkan,
            points: &points_elkan,
            iterations_t: 4,
            label: "test_elkan".to_string(),
            compute_rms: true,
        };
        let cluster_args_original = ClusterArgs {
            algorithm: ClusterAlgorithm::KmeansOriginal,
            init_centers: &init_centers_original,
            points: &points_original,
            label: "test_original".to_string(),
            ..cluster_args_elkan
        };

        let (_, all_rms_elkan) = cluster(&clusterable, &cluster_args_elkan);
        let (_, all_rms_original) = cluster(&clusterable, &cluster_args_original);
        assert_eq!(all_rms_elkan.len(), 4);
        assert_eq!(all_rms_original.len(), 4);

        for (elkan2003_rms, original_rms) in all_rms_elkan.iter().zip(all_rms_original) {
            println!("elkan: {}, original: {}", elkan2003_rms, original_rms);
            assert!(
                (elkan2003_rms - original_rms).abs() < 0.00001,
                "RMS-es (elkan: {}, original: {}) should approximately match at each step",
                elkan2003_rms,
                original_rms
            )
        }
    }
}
