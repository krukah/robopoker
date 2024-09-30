use crate::clustering::abstraction::Abstraction;
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
    fn distance(&self, x: &Abstraction, y: &Abstraction) -> f32;
}

impl Metric for BTreeMap<Pair, f32> {
    /// Earth Mover's Distance (EMD) between histograms
    ///
    /// This function approximates the Earth Mover's Distance (EMD) between two histograms.
    /// EMD is a measure of the distance between two probability distributions.
    /// It is calculated by finding the minimum amount of "work" required to transform
    /// one distribution into the other.
    ///
    /// Beware the asymmetry:
    /// EMD(X,Y) != EMD(Y,X)
    /// Centroid should be the "hole" (sink) in the EMD calculation
    fn emd(&self, source: &Histogram, target: &Histogram) -> f32 {
        let x = source.domain();
        let y = target.domain();
        let mut energy = 0.0;
        let mut hasmoved = x
            .iter()
            .map(|&a| (a, false))
            .collect::<BTreeMap<&Abstraction, bool>>();
        let mut notmoved = x
            .iter()
            .map(|&a| (a, 1.0 / x.len() as f32))
            .collect::<BTreeMap<&Abstraction, f32>>();
        let mut unfilled = y
            .iter()
            .map(|&a| (a, target.weight(a)))
            .collect::<BTreeMap<&Abstraction, f32>>(); // this is effectively a clone
        for _ in 0..y.len() {
            for pile in x.iter() {
                // skip if we have already moved all the earth from this source
                if *hasmoved.get(pile).expect("in x domain") {
                    continue;
                }
                // find the nearest neighbor of X (source) from Y (sink)
                let (hole, distance) = y
                    .iter()
                    .map(|sink| (*sink, self.distance(pile, sink)))
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).expect("not NaN"))
                    .expect("y domain not empty");
                // decide if we can remove earth from both distributions
                let demand = *notmoved.get(pile).expect("in x domain");
                let vacant = *unfilled.get(hole).expect("in y domain");
                if vacant > 0.0 {
                    energy += distance * demand.min(vacant);
                } else {
                    continue;
                }
                // remove earth from both distributions
                if demand > vacant {
                    *notmoved.get_mut(pile).expect("in x domain") -= vacant;
                    *unfilled.get_mut(hole).expect("in y domain") = 0.0;
                } else {
                    *hasmoved.get_mut(pile).expect("in x domain") = true;
                    *notmoved.get_mut(pile).expect("in x domain") = 0.0;
                    *unfilled.get_mut(hole).expect("in y domain") -= demand;
                }
            }
        }
        energy
    }

    /// generated recursively and hierarchically
    /// we can calculate the distance between two abstractions
    /// by eagerly finding distance between their centroids
    fn distance(&self, x: &Abstraction, y: &Abstraction) -> f32 {
        match (x, y) {
            (Abstraction::Equity(a), Abstraction::Equity(b)) => (a - b).abs() as f32,
            (Abstraction::Random(_), Abstraction::Random(_)) => {
                let ref xor = Pair::from((x, y));
                self.get(xor).copied().expect("precalculated distance")
            }
            _ => unreachable!("invalid abstraction pair"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::cards::strength::Strength;
    use crate::clustering::histogram::Histogram;

    #[tokio::test]
    async fn test_random_streets_emd() {
        let obs1 = Observation::from(Street::Turn);
        let obs2 = Observation::from(Street::Turn);
        let ref h1 = Histogram::from(obs1.clone());
        let ref h2 = Histogram::from(obs2.clone());
        println!("{}\n{} {}", h1, Strength::from(obs1.clone()), obs1);
        println!("{}\n{} {}", h2, Strength::from(obs2.clone()), obs2);
        println!();
        println!("EMD A >> B: {}", BTreeMap::new().emd(h1, h2)); ////////
        println!("EMD B >> A: {}", BTreeMap::new().emd(h2, h1)); ////////
    }
}
