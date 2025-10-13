use super::*;
use crate::Energy;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub trait KMeans: Sync {
    type P: Absorb + Send + Sync + Clone;

    fn distance(&self, h1: &Self::P, h2: &Self::P) -> Energy;

    fn t(&self) -> usize;
    fn k(&self) -> usize;
    fn n(&self) -> usize;

    fn points(&self) -> &Vec<Self::P>;
    fn kmeans(&self) -> &Vec<Self::P>;
    fn boundaries(&self) -> &Vec<Bound>;

    fn datum(&self, i: usize) -> &Self::P {
        self.points().get(i).expect("n bounds")
    }
    fn kmean(&self, j: usize) -> &Self::P {
        self.kmeans().get(j).expect("k bounds")
    }
    fn bound(&self, i: usize) -> &Bound {
        self.boundaries().get(i).expect("n bounds")
    }

    /// Compute the nearest neighbor in O(k) * MetricCost
    fn neighbor(&self, i: usize) -> (usize, f32) {
        let ref x = self.datum(i);
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, self.distance(c, x)))
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
            .unwrap()
    }

    /// Compute d(c, c') for all centers c and c'
    fn pairwise(&self) -> Vec<Vec<f32>> {
        self.kmeans()
            .iter()
            .flat_map(|c1| self.kmeans().iter().map(|c2| self.distance(c1, c2)))
            .collect::<Vec<_>>()
            .chunks(self.k())
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>()
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
                    .unwrap()
            })
            .collect::<Vec<_>>()
    }

    /// Identify points where u(x) <= s(c(x))
    fn excluded(&self) -> HashSet<usize> {
        let ref midpoints = self.midpoints();
        self.boundaries()
            .iter()
            .enumerate()
            .filter(|(_, bs)| bs.u() <= midpoints[bs.j()])
            .map(|(x, _)| x)
            .collect::<HashSet<_>>()
    }

    /// Identify points where u(x) > s(c(x)) requiring bound updates
    fn included(&self) -> HashMap<usize, (&Self::P, Bound)> {
        let ref exclusions = self.excluded();
        (0..self.n())
            .filter(|i| !exclusions.contains(i))
            .map(|i| (i, (self.datum(i), self.bound(i))))
            .map(|(i, (p, b))| (i, (p, b.clone())))
            .collect::<HashMap<_, _>>()
    }

    /// Step 3: Update bounds for each point/center pair using triangle inequality
    fn updating(&self) -> HashMap<usize, (&Self::P, Bound)> {
        let ref pairwise = self.pairwise();
        let mut included = self.included();
        (0..self.k()).for_each(|j| {
            included
                .par_iter_mut()
                .map(|(_, (p, b))| (p, b))
                .filter(|(_, b)| b.needs_update(j, pairwise))
                .for_each(|(p, b)| self.update(pairwise, p, b, j));
        });
        included
    }

    fn update(&self, pairs: &[Vec<Energy>], p: &Self::P, b: &mut Bound, j: usize) {
        b.update(j, pairs, |j| self.distance(p, self.kmean(j)))
    }

    /// Merge updated bounds back with original
    fn next_boundaries(&self) -> Vec<Bound> {
        let ref new = self.updating();
        self.boundaries()
            .par_iter()
            .enumerate()
            .map(|(i, original)| new.get(&i).map(|(_, b)| b).unwrap_or_else(|| original))
            .cloned()
            .collect::<Vec<_>>()
    }

    fn next_centroids(&self) -> Vec<Self::P> {
        (0..self.k())
            .map(|j| {
                self.boundaries() // if this is the first iteration, we should sample k() random points from dataset
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.j == j)
                    .map(|(i, _)| self.datum(i))
                    .fold(Self::P::default(), Self::P::absorb)
            })
            .collect::<Vec<_>>()
    }

    fn movements(&self, centroids: &[Self::P]) -> Vec<Energy> {
        assert!(centroids.len() == self.k());
        self.kmeans()
            .par_iter()
            .zip(centroids.par_iter())
            .map(|(old, new)| self.distance(new, old))
            .collect::<Vec<_>>()
    }

    /// Compute new centroids from assigned points
    fn next(&self) -> (Vec<Self::P>, Vec<Bound>) {
        let centroids = self.next_centroids();
        let ref movements = self.movements(&centroids);
        let boundaries = self
            .next_boundaries()
            .into_par_iter()
            .map(|b| b.update_lower(movements))
            .map(|b| b.update_upper(movements))
            .collect::<Vec<_>>();
        (centroids, boundaries)
    }
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
