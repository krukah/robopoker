use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::xor::Pair;
use std::collections::HashMap;

/// Trait for defining distance metrics between abstractions and histograms.
///
/// Calculating similarity between abstractions
/// and Earth Mover's Distance (EMD) between histograms. These metrics are
/// essential for clustering algorithms and comparing distributions.
pub trait Metric {
    fn emd(&self, x: &Histogram, y: &Histogram) -> f32;
    fn distance(&self, x: &Abstraction, y: &Abstraction) -> f32;
}

impl Metric for HashMap<Pair, f32> {
    fn emd(&self, x: &Histogram, y: &Histogram) -> f32 {
        let x_domain = x.domain();
        let y_domain = y.domain();
        let n = x_domain.len();
        let m = y_domain.len();
        let mut energy = 0.0;
        let mut completed = x_domain
            .iter()
            .map(|&a| (a, false))
            .collect::<HashMap<&Abstraction, bool>>();
        let mut pressures = x_domain
            .iter()
            .map(|&a| (a, 1.0 / n as f32))
            .collect::<HashMap<&Abstraction, f32>>();
        let mut vacancies = y_domain
            .iter()
            .map(|&a| (a, y.weight(a)))
            .collect::<HashMap<&Abstraction, f32>>(); // this is effectively a clone
        for _ in 0..m {
            for from in x_domain.iter() {
                // skip if we have already moved all the earth from this source
                if *completed.get(from).expect("from in spent domain") {
                    continue;
                }
                // find the nearest neighbor of X (source) from Y (sink)
                let (ref into, nearest) = y
                    .domain()
                    .iter()
                    .map(|mean| (*mean, self.distance(from, mean)))
                    .min_by(|&(_, ref a), &(_, ref b)| a.partial_cmp(b).expect("not NaN"))
                    .expect("y domain is not empty");
                let demand = *pressures.get(from).expect("from in x domain");
                let vacant = *vacancies.get(into).expect("into in y domain");
                // decide if we can remove earth from both distributions
                if vacant > 0.0 {
                    energy += nearest * demand.min(vacant);
                } else {
                    continue;
                }
                // remove earth from both distributions
                if demand > vacant {
                    *pressures.get_mut(from).expect("from in x domain") -= vacant;
                    *vacancies.get_mut(into).expect("into in y domain") = 0.0;
                } else {
                    *completed.get_mut(from).expect("from in x domain") = true;
                    *pressures.get_mut(from).expect("from in x domain") = 0.0;
                    *vacancies.get_mut(into).expect("into in y domain") -= demand;
                }
            }
        }
        energy
    }
    fn distance(&self, x: &Abstraction, y: &Abstraction) -> f32 {
        let ref xor = Pair::from((x, y));
        *self.get(xor).expect("precalculated distance")
    }
}
