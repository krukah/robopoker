use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::progress::Progress;
use crate::clustering::xor::Pair;
use std::collections::BTreeMap;

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default)]
pub struct Metric(pub BTreeMap<Pair, f32>);

impl Metric {
    /// This function *approximates* the Earth Mover's Distance (EMD) between two histograms.
    /// EMD is a measure of the distance between two probability distributions.
    /// It is calculated by finding the minimum amount of "work" required to transform
    /// one distribution into the other.
    ///
    /// for Histogram<T> where T: Ord, we can efficiently and accurately calculate EMD
    /// 1. sort elements of the support (already done for free because BTreeMap)
    /// 2. produce CDF of source and target distributions
    /// 3. integrate difference between CDFs over support
    ///
    /// we only have the luxury of this efficient O(N) calculation on Street::Turn,
    /// where the support is over the Abstraction::Equity(i8) variant.
    pub fn emd(&self, source: &Histogram, target: &Histogram) -> f32 {
        match target.peek() {
            Abstraction::Equity(_) => Self::difference(source, target),
            Abstraction::Random(_) => self.wasserstein(source, target),
        }
    }

    /// generated recursively and hierarchically
    /// we can calculate the distance between two abstractions
    /// by eagerly finding distance between their centroids
    fn distance(&self, x: &Abstraction, y: &Abstraction) -> f32 {
        match (x, y) {
            (Abstraction::Equity(a), Abstraction::Equity(b)) => (a - b).abs() as f32,
            (Abstraction::Random(_), Abstraction::Random(_)) => self
                .0
                .get(&Pair::from((x, y)))
                .copied()
                .expect("precalculated distance"),
            _ => unreachable!("invalid abstraction pair"),
        }
    }

    /// here we have the luxury of calculating EMD
    /// over 1-dimensional support of Abstraction::Equity
    /// so we just integrate the absolute difference between CDFs
    fn difference(x: &Histogram, y: &Histogram) -> f32 {
        let mut total = 0.;
        let mut cdf_x = 0.;
        let mut cdf_y = 0.;
        for abstraction in Abstraction::range() {
            cdf_x += x.weight(abstraction);
            cdf_y += y.weight(abstraction);
            total += (cdf_x - cdf_y).abs();
        }
        total
    }

    /// Beware the asymmetry:
    /// EMD(X,Y) != EMD(Y,X)
    /// Centroid should be the "hole" (sink) in the EMD calculation
    fn wasserstein(&self, source: &Histogram, target: &Histogram) -> f32 {
        let x = source.support();
        let y = target.support();
        let mut energy = 0.;
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
                if vacant > 0. {
                    energy += distance * demand.min(vacant);
                } else {
                    continue;
                }
                // remove earth from both distributions
                if demand > vacant {
                    *notmoved.get_mut(pile).expect("in x domain") -= vacant;
                    *unfilled.get_mut(hole).expect("in y domain") = 0.;
                } else {
                    *unfilled.get_mut(hole).expect("in y domain") -= demand;
                    *notmoved.get_mut(pile).expect("in x domain") = 0.;
                    *hasmoved.get_mut(pile).expect("in x domain") = true;
                }
            }
        }
        energy
    }

    /// save profile to disk in a PGCOPY compatible format
    pub fn save(&self, path: String) {
        log::info!("uploading abstraction metric {}", path);
        use byteorder::BigEndian;
        use byteorder::WriteBytesExt;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.metric.pgcopy", path)).expect("new file");
        let ref mut progress = Progress::new(self.0.len(), 10);
        file.write_all(b"PGCOPY\n\xff\r\n\0").expect("header");
        file.write_u32::<BigEndian>(0).expect("flags");
        file.write_u32::<BigEndian>(0).expect("extension");
        for (pair, distance) in self.0.iter() {
            file.write_u16::<BigEndian>(2).expect("field count");
            file.write_u32::<BigEndian>(8).expect("8-bytes field");
            file.write_i64::<BigEndian>(i64::from(*pair)).expect("pair");
            file.write_u32::<BigEndian>(4).expect("4-bytes field");
            file.write_f32::<BigEndian>(*distance).expect("distance");
            progress.tick();
        }
        file.write_u16::<BigEndian>(0xFFFF).expect("trailer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::clustering::histogram::Histogram;

    #[test]
    fn is_histogram_emd_zero() {
        let metric = Metric::default();
        let obs = Observation::from(Street::Turn);
        let ref h1 = Histogram::from(obs.clone());
        let ref h2 = Histogram::from(obs.clone());
        assert!(metric.emd(h1, h2) == 0.);
        assert!(metric.emd(h2, h1) == 0.);
    }

    #[test]
    fn is_histogram_emd_positive() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        assert!(metric.emd(h1, h2) > 0.);
        assert!(metric.emd(h2, h1) > 0.);
    }

    #[test]
    fn is_histogram_emd_symmetric() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        assert!(metric.emd(h1, h2) == metric.emd(h2, h1));
    }
}
