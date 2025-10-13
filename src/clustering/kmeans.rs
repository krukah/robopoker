use super::*;
use crate::Energy;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub trait Elkan: Sync {
    type P: Absorb + Sync;

    fn distance(&self, h1: &Self::P, h2: &Self::P) -> Energy;

    fn dataset(&self) -> &Vec<Self::P>;
    fn centers(&self) -> &Vec<Self::P>;
    fn boundaries(&self) -> &Vec<Bound>;

    fn step(&mut self);

    fn t(&self) -> usize {
        1024
    }
    fn k(&self) -> usize {
        self.centers().len()
    }
    fn n(&self) -> usize {
        self.dataset().len()
    }

    fn point(&self, i: usize) -> &Self::P {
        self.dataset().get(i).expect("n bounds")
    }
    fn kmean(&self, j: usize) -> &Self::P {
        self.centers().get(j).expect("k bounds")
    }
    fn bound(&self, i: usize) -> &Bound {
        self.boundaries().get(i).expect("n bounds")
    }

    /// Compute the nearest neighbor in O(k) * MetricCost
    fn neighbor(&self, i: usize) -> (usize, f32) {
        let ref x = self.point(i);
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
            .flat_map(|c1| self.centers().iter().map(|c2| self.distance(c1, c2)))
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
            // .iter()
            .par_iter()
            .enumerate()
            .filter(|(_, b)| b.u() <= midpoints[b.j()])
            .map(|(x, _)| x)
            .collect::<HashSet<_>>()
    }

    /// Identify points where u(x) > s(c(x)) requiring bound updates
    fn triangle(&self) -> HashMap<usize, (&Self::P, Bound)> {
        let ref exclusions = self.excluded();
        (0..self.n())
            .filter(|i| !exclusions.contains(i))
            .map(|i| (i, (self.point(i), self.bound(i))))
            .map(|(i, (p, b))| (i, (p, b.clone())))
            .collect::<HashMap<_, _>>()
    }

    /// Step 3: Update bounds for each point/center pair using triangle inequality
    fn updating(&self) -> HashMap<usize, (&Self::P, Bound)> {
        let ref pairwise = self.pairwise();
        let mut included = self.triangle();
        (0..self.k()).for_each(|j| {
            included
                .par_iter_mut()
                .map(|(_, (x, b))| (x, b))
                .filter(|(_, b)| b.needs_update(j, pairwise))
                .for_each(|(x, b)| self.modify(pairwise, x, b, j));
        });
        included
    }

    fn modify(&self, pairs: &[Vec<Energy>], p: &Self::P, b: &mut Bound, j: usize) {
        b.update(j, pairs, |j| self.distance(p, self.kmean(j)))
    }

    /// Merge updated bounds back with original
    fn next_bounds(&self) -> Vec<Bound> {
        let ref new = self.updating();
        self.boundaries()
            .par_iter()
            .enumerate()
            .map(|(i, original)| new.get(&i).map(|(_, b)| b).unwrap_or_else(|| original))
            .cloned()
            .collect::<Vec<_>>()
    }

    fn next_kmeans(&self) -> Vec<Self::P> {
        (0..self.k())
            .map(|j| {
                self.boundaries()
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.j == j)
                    .map(|(i, _)| self.point(i))
                    .fold(Self::P::default(), Self::P::absorb)
            })
            .collect::<Vec<_>>()
    }

    fn gradient(&self, news: &[Self::P]) -> Vec<Energy> {
        assert!(news.len() == self.k());
        self.centers()
            .par_iter()
            .zip(news.par_iter())
            .map(|(old, new)| self.distance(new, old))
            .collect::<Vec<_>>()
    }

    /// Compute new centroids from assigned points
    fn next(&self) -> (Vec<Self::P>, Vec<Bound>) {
        let kmeans = self.next_kmeans();
        let ref gradient = self.gradient(&kmeans);
        let bounds = self
            .next_bounds()
            .into_par_iter()
            .map(|b| b.shift(gradient))
            .collect::<Vec<_>>();
        (kmeans, bounds)
    }

    fn rms(&self) -> Energy {
        (self
            .boundaries()
            .par_iter()
            .enumerate()
            .map(|(i, b)| self.distance(self.point(i), self.kmean(b.j())))
            .map(|d| d * d)
            .sum::<Energy>()
            / self.n() as Energy)
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::clustering::histogram::Histogram;
    use crate::clustering::metric::Metric;

    /// it just so happens that we can cluster arbitrary
    /// subsets of Turn Histogram equity distributions,
    /// because we project into the River space on the fly (obs.equity())
    struct Turns {
        metric: Metric,
        dataset: Vec<Histogram>,
        centers: Vec<Histogram>,
        boundaries: Vec<Bound>,
    }

    impl Turns {
        const K: usize = 4;
        const N: usize = 64;
        const T: usize = 8;

        fn new() -> Self {
            let k = Self::K;
            let n = Self::N;
            let metric = Metric::default();
            let dataset = (0..n)
                .map(|_| Histogram::from(Observation::from(Street::Turn)))
                .collect();
            let centers = (0..k)
                .map(|_| Histogram::from(Observation::from(Street::Turn)))
                .collect();
            let mut km = Self {
                metric,
                dataset,
                centers,
                boundaries: vec![],
            };
            km.boundaries = (0..n)
                .map(|i| km.neighbor(i))
                .map(|(j, d)| Bound::new(j, k, d))
                .collect::<Vec<_>>();
            km
        }
    }

    impl Elkan for Turns {
        type P = Histogram;
        fn t(&self) -> usize {
            Self::T
        }
        fn dataset(&self) -> &Vec<Histogram> {
            &self.dataset
        }
        fn centers(&self) -> &Vec<Histogram> {
            &self.centers
        }
        fn boundaries(&self) -> &Vec<Bound> {
            &self.boundaries
        }
        fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
            self.metric.emd(h1, h2)
        }
        fn step(&mut self) {
            let (centers, boundaries) = self.next();
            self.centers = centers;
            self.boundaries = boundaries;
        }
    }

    #[test]
    fn elkan_rms_decreases() {
        let mut km = Turns::new();
        let mut rms = vec![km.rms()];
        (0..km.t()).for_each(|_| {
            km.step();
            println!("RMS: {}", km.rms());
            rms.push(km.rms());
        });
        for window in rms.windows(2) {
            assert!(
                window[0] >= window[1],
                "RMS increasing: {} -> {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn elkan_rms_converges() {
        let mut km = Turns::new();
        (0..km.t()).for_each(|_| km.step());
        let r1 = km.rms();
        km.step();
        let r2 = km.rms();
        println!("RMS: {} -> {}", r1, r2);
        assert!(
            (r1 - r2).abs() <= 0.005,
            "RMS not converged: {} -> {}",
            r1,
            r2
        );
    }

    #[test]
    fn elkan_assigns_all_points() {
        let km = Turns::new();
        let assignments = km
            .boundaries()
            .iter()
            .map(|b| b.j())
            .collect::<HashSet<_>>();
        assert!(assignments.len() > 0);
        assert!(assignments.iter().all(|&j| j < km.k()));
    }
}
