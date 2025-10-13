use super::*;
use crate::Energy;
use rayon::prelude::*;
use std::collections::*;

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
            .chunks(self.k())
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
            .filter(|(_, bs)| bs.upper <= midpoints[bs.j])
            .map(|(x, _)| x)
            .collect()
    }

    /// Identify points where u(x) > s(c(x)) requiring bound updates
    fn inclusions(&self) -> HashMap<usize, (&Self::P, Bounds)> {
        let exclusions = self.exclusions();
        self.points()
            .iter()
            .enumerate()
            .filter(|(i, _)| !exclusions.contains(i))
            .map(|(i, p)| (i, (p, self.metadata()[i].clone())))
            .collect()
    }

    /// Step 3: Update bounds for each point/center pair using triangle inequality
    fn boundaries(&self) -> HashMap<usize, (&Self::P, Bounds)> {
        let ref pairwises = self.pairwise();
        let mut inclusions = self.inclusions();
        for j in 0..self.k() {
            inclusions
                .par_iter_mut()
                .filter(|(_, (_, bs))| bs.should_update(j, pairwises))
                .for_each(|(_, (p, bounds))| {
                    bounds.update(j, pairwises, |idx| self.distance(p, &self.centers()[idx]))
                });
        }
        inclusions
    }

    /// Merge updated bounds back with original
    fn merge(&self) -> Vec<Bounds> {
        let boundaries = self.boundaries();
        self.metadata()
            .iter()
            .enumerate()
            .map(|(i, og)| {
                boundaries
                    .get(&i)
                    .map(|(_, bs)| bs.clone())
                    .unwrap_or_else(|| og.clone())
            })
            .collect()
    }

    /// Group points by assigned centroid
    fn groupings(&self) -> (Vec<Vec<&Self::P>>, Vec<Bounds>) {
        let merged = self.merge();
        let assignments = self
            .centers()
            .iter()
            .enumerate()
            .map(|(j, _)| {
                merged
                    .iter()
                    .enumerate()
                    .filter(|(_, helper)| helper.j == j)
                    .map(|(i, _)| &self.points()[i])
                    .collect()
            })
            .collect();
        (assignments, merged)
    }

    /// Compute new centroids from assigned points
    fn solution(&self) -> (Vec<Self::P>, Vec<Bounds>) {
        let (assignments, merged) = self.groupings();
        let centroids = assignments
            .into_iter()
            .map(|group| group.into_iter().fold(Self::P::default(), Self::P::absorb))
            .collect();
        (centroids, merged)
    }

    fn next(&self) -> (Vec<Self::P>, Vec<Bounds>) {
        let (centroids, boundaries) = self.solution();
        let movements = centroids
            .par_iter()
            .zip(self.centers().par_iter())
            .map(|(new, old)| self.distance(old, new))
            .collect::<Vec<_>>();
        let boundaries = boundaries
            .into_par_iter()
            .map(|helper| helper.update_lower(&movements))
            .map(|helper| helper.update_upper(&movements))
            .collect::<Vec<_>>();
        (centroids, boundaries)
    }
}

/// Initialize metadata using triangle inequality (Elkan lemma 1)
fn init_metadata() -> Vec<(usize, f32)> {
    todo!("Initialize metadata: Vec<(usize, f32)> using Elkan 2003 lemma 1 for efficient nearest neighbor assignment")
}

#[cfg(feature = "native")]
/// Legacy cluster function - use KMeans::cluster() instead
pub fn cluster<T: KMeans + std::marker::Sync>(
    clusterable: &T,
    cluster_args: &ClusterArgs<T::P>,
) -> (Vec<T::P>, Vec<f32>) {
    todo!("Use KMeans::cluster() trait method instead")
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::Arbitrary;
//     use crate::Energy;

//     #[derive(Debug)]
//     struct MockClusterable {}

//     fn create_seeded_histograms(i: i32) -> Vec<Histogram> {
//         (0..i).map(|_| Histogram::random()).collect()
//     }

//     #[test]
//     fn test_kmeans_elkan2003_rms_converges() {
//         let points: Vec<Histogram> = create_seeded_histograms(400);
//         let init_centers: Vec<Histogram> = create_seeded_histograms(2);

//         let clusterable = MockClusterable {};
//         let cluster_args = ClusterArgs {
//             algorithm: ClusterAlgorithm::KmeansElkan2003,
//             init_centers: &init_centers,
//             points: &points,
//             iterations_t: 6,
//             label: "test_elkan2003_converges".to_string(),
//             compute_rms: true,
//         };

//         let (result, all_rms) = cluster(&clusterable, &cluster_args);
//         for w in all_rms.windows(2) {
//             println!("{} {}", w[0], w[1]);
//         }

//         // "Safety" checks to make sure things are overall behaving as expected
//         assert_eq!(result.len(), 2);
//         assert_eq!(all_rms.len(), 6);
//         assert!(
//             (all_rms[0] - all_rms[1]).abs() > 0.01,
//             "RMS was already converged too much for a fair test at the start (goes from {} to {})",
//             all_rms[0],
//             all_rms[1]
//         );

//         // Actual asserts the test is targeting ("did the RMS converge near the end")
//         for w in all_rms.into_iter().skip(3).collect::<Vec<_>>().windows(2) {
//             let prior_rms = w[0];
//             let next_rms = w[1];
//             println!("{} {}", prior_rms, next_rms);

//             assert!(
//                 (prior_rms - next_rms).abs() <= 0.0005,
//                 "RMS is still decreasing _too much_ / did not converge enough (goes from {} to {})",
//                 prior_rms,
//                 next_rms
//             );
//         }
//     }

//     #[test]
//     fn test_kmeans_elkan2003_rms_decreases() {
//         let points: Vec<Histogram> = create_seeded_histograms(500);
//         let init_centers: Vec<Histogram> = create_seeded_histograms(5);

//         let clusterable = MockClusterable {};
//         let cluster_args = ClusterArgs {
//             algorithm: ClusterAlgorithm::KmeansElkan2003,
//             init_centers: &init_centers,
//             points: &points,

//             // Don't set too high; the values stop decreasing as much in normal operation once it starts converging.
//             iterations_t: 4,

//             label: "test_elkan2003_decreases".to_string(),
//             compute_rms: true,
//         };

//         let (result, all_rms) = cluster(&clusterable, &cluster_args);
//         assert_eq!(result.len(), 5);
//         assert_eq!(all_rms.len(), 4);

//         for w in all_rms.windows(2) {
//             let prior_rms = w[0];
//             let next_rms = w[1];
//             println!("{} {}", prior_rms, next_rms);

//             assert!(
//                 next_rms < prior_rms,
//                 "RMS was not monotonically decreasing (goes from {} to {})",
//                 prior_rms,
//                 next_rms
//             );
//             assert!(
//                 (prior_rms - next_rms).abs() > 0.0001,
//                 "RMS did not decrease *enough* during at least one iteration (goes from {} to {})",
//                 prior_rms,
//                 next_rms
//             );
//         }
//     }

//     #[test]
//     fn test_kmeans_original_rms_decreases() {
//         let points: Vec<Histogram> = create_seeded_histograms(400);
//         let init_centers: Vec<Histogram> = create_seeded_histograms(5);
//         let clusterable = MockClusterable {};
//         let cluster_args = ClusterArgs {
//             algorithm: ClusterAlgorithm::KmeansOriginal,
//             init_centers: &init_centers,
//             points: &points,

//             // Don't set too high; the values stop decreasing as much in normal operation once it starts converging
//             iterations_t: 4,

//             label: "test_original".to_string(),
//             compute_rms: true,
//         };

//         let (result, all_rms) = cluster(&clusterable, &cluster_args);
//         assert_eq!(result.len(), 5);
//         assert_eq!(all_rms.len(), 4);

//         for w in all_rms.windows(2) {
//             let prior_rms = w[0];
//             let next_rms = w[1];
//             assert!(
//                 next_rms < prior_rms,
//                 "RMS was not monotonially decreasing (goes from {} to {})",
//                 prior_rms,
//                 next_rms
//             );
//             assert!(
//                 (prior_rms - next_rms).abs() > 0.0001,
//                 "RMS did not decrease *enough* during at least one iteration (goes from {} to {})",
//                 prior_rms,
//                 next_rms
//             );
//             println!("{} {}", prior_rms, next_rms);
//         }
//     }

//     /// As per the research paper:
//     /// "After each iteration, [Elkan's algorithm] produces the same set of center locations as the standard k-means method."
//     /// Therefore, the RMS we compute at every single iteration should be (nearly) identical.
//     #[test]
//     fn test_kmeans_elkan2003_original_rms_matches() {
//         let points_elkan: Vec<Histogram> = create_seeded_histograms(500);
//         let points_original: Vec<Histogram> = points_elkan.clone();
//         let init_centers_elkan: Vec<Histogram> = create_seeded_histograms(3);
//         let init_centers_original: Vec<Histogram> = init_centers_elkan.clone();
//         let clusterable = MockClusterable {};
//         let cluster_args_elkan = ClusterArgs {
//             algorithm: ClusterAlgorithm::KmeansElkan2003,
//             init_centers: &init_centers_elkan,
//             points: &points_elkan,
//             iterations_t: 4,
//             label: "test_elkan".to_string(),
//             compute_rms: true,
//         };
//         let cluster_args_original = ClusterArgs {
//             algorithm: ClusterAlgorithm::KmeansOriginal,
//             init_centers: &init_centers_original,
//             points: &points_original,
//             label: "test_original".to_string(),
//             ..cluster_args_elkan
//         };

//         let (_, all_rms_elkan) = cluster(&clusterable, &cluster_args_elkan);
//         let (_, all_rms_original) = cluster(&clusterable, &cluster_args_original);
//         assert_eq!(all_rms_elkan.len(), 4);
//         assert_eq!(all_rms_original.len(), 4);

//         for (elkan2003_rms, original_rms) in all_rms_elkan.iter().zip(all_rms_original) {
//             println!("elkan: {}, original: {}", elkan2003_rms, original_rms);
//             assert!(
//                 (elkan2003_rms - original_rms).abs() < 0.00001,
//                 "RMS-es (elkan: {}, original: {}) should approximately match at each step",
//                 elkan2003_rms,
//                 original_rms
//             )
//         }
//     }
// }
