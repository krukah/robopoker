use crate::clustering::abstraction::NodeAbstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::xor::Pair;
use std::collections::BTreeMap;

/// Trait for defining distance metrics between abstractions and histograms.
///
/// Calculating similarity between abstractions
/// and Earth Mover's Distance (EMD) between histograms. These metrics are
/// essential for clustering algorithms and comparing distributions.
pub trait Metric {
    fn emd(&self, x: &Histogram, y: &Histogram) -> f32;
    fn distance(&self, x: &NodeAbstraction, y: &NodeAbstraction) -> f32;
}

impl Metric for BTreeMap<Pair, f32> {
    fn emd(&self, x: &Histogram, y: &Histogram) -> f32 {
        let x_domain = x.domain();
        let y_domain = y.domain();
        let n = x_domain.len();
        let m = y_domain.len();
        let mut energy = 0.0;
        let mut completed = x_domain
            .iter()
            .map(|&a| (a, false))
            .collect::<BTreeMap<&NodeAbstraction, bool>>();
        let mut pressures = x_domain
            .iter()
            .map(|&a| (a, 1.0 / n as f32))
            .collect::<BTreeMap<&NodeAbstraction, f32>>();
        let mut vacancies = y_domain
            .iter()
            .map(|&a| (a, y.weight(a)))
            .collect::<BTreeMap<&NodeAbstraction, f32>>(); // this is effectively a clone
        for _ in 0..m {
            for source in x_domain.iter() {
                // skip if we have already moved all the earth from this source
                if *completed.get(source).expect("in x domain") {
                    continue;
                }
                // find the nearest neighbor of X (source) from Y (sink)
                let (ref drains, nearest) = y_domain
                    .iter()
                    .map(|mean| (*mean, self.distance(source, mean)))
                    .min_by(|&(_, ref a), &(_, ref b)| a.partial_cmp(b).expect("not NaN"))
                    .expect("y domain not empty");
                let demand = *pressures.get(source).expect("in x domain");
                let vacant = *vacancies.get(drains).expect("in y domain");
                // decide if we can remove earth from both distributions
                if vacant > 0.0 {
                    energy += nearest * demand.min(vacant);
                } else {
                    continue;
                }
                // remove earth from both distributions
                if demand > vacant {
                    *pressures.get_mut(source).expect("in x domain") -= vacant;
                    *vacancies.get_mut(drains).expect("in y domain") = 0.0;
                } else {
                    *completed.get_mut(source).expect("in x domain") = true;
                    *pressures.get_mut(source).expect("in x domain") = 0.0;
                    *vacancies.get_mut(drains).expect("in y domain") -= demand;
                }
            }
        }
        energy
    }
    fn distance(&self, x: &NodeAbstraction, y: &NodeAbstraction) -> f32 {
        let ref xor = Pair::from((x, y));
        self.get(xor).copied().expect("precalculated distance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::NodeObservation;
    use crate::cards::street::Street;
    use crate::cards::strength::Strength;
    use crate::clustering::histogram::Histogram;
    use crate::clustering::layer::Layer;
    use rand::seq::SliceRandom;
    /// select two random Turn hands
    /// and calculate the EMD between them
    /// not apples to apples but results should be intuitive
    /// (?)
    #[tokio::test]
    async fn test_random_streets_emd() {
        let obs1 = NodeObservation::from(Street::Turn);
        let obs2 = NodeObservation::from(Street::Turn);
        let ref h1 = Histogram::from(obs1.clone());
        let ref h2 = Histogram::from(obs2.clone());
        println!("{} {}\n{}", Strength::from(obs1.clone()), obs1, h1);
        println!("{} {}\n{}", Strength::from(obs2.clone()), obs2, h2);
        println!("EMD A << B: {}", Layer::outer_metric().emd(h1, h2));
        println!("EMD B << A: {}", Layer::outer_metric().emd(h2, h1));
    }

    #[tokio::test]
    async fn test_random_pair_symmetry() {
        let ref mut rng = rand::thread_rng();
        let metric = Layer::outer_metric();
        let histo = Histogram::from(NodeObservation::from(Street::Turn));
        let ref pair = histo
            .domain()
            .choose_multiple(rng, 2)
            .cloned()
            .collect::<Vec<_>>();
        let d1 = metric.distance(pair[0], pair[1]);
        let d2 = metric.distance(pair[1], pair[0]);
        assert!(d1 == d2,);
    }
}
