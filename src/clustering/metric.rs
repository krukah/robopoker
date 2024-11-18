use super::equity::Equity;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::xor::Pair;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use std::collections::BTreeMap;

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default)]
pub struct Metric(pub BTreeMap<Pair, f32>);

impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32 {
        match (x, y) {
            (Self::X::Unique(_), Self::Y::Unique(_)) => self.lookup(x, y),
            (Self::X::Equity(_), Self::Y::Equity(_)) => Equity.distance(x, y),
            (Self::X::Pocket(_), Self::Y::Pocket(_)) => unreachable!("no preflop distance"),
            _ => unreachable!(),
        }
    }
}

impl Coupling for Metric {
    type M = Self;
    type X = Abstraction;
    type Y = Abstraction;
    type P = Histogram;
    type Q = Histogram;
    fn flow(&self, _: &Self::X, _: &Self::Y) -> f32 {
        todo!("implementation would require us to eagerly calculate and store P_ij in Metric constructor(s). e.g. if we use a LP solver to get an exact solution, then Metric would store matrix elements of the optimal P_ij transport plan.")
    }
    fn cost(&self, x: &Self::P, y: &Self::Q, _: &Self::M) -> f32 {
        self.emd(x, y)
    }
}

impl Metric {
    pub fn emd(&self, source: &Histogram, target: &Histogram) -> f32 {
        match source.peek() {
            Abstraction::Unique(_) => self.greedy(source, target),
            Abstraction::Equity(_) => Equity::variation(source, target),
            Abstraction::Pocket(_) => unreachable!("no preflop emd"),
        }
    }
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> f32 {
        if x == y {
            0.
        } else {
            *self.0.get(&Pair::from((x, y))).unwrap()
        }
    }

    /// greedy algorithm for optimimal transport.
    /// my favorite interpretation of this is in the formalization
    /// of bipartite matching. we have a Left set of sources and a
    /// Right set of targets. we want to find a way to pair each source
    /// to a target under the constraint of conserving probability mass.
    ///
    /// for each step of the algorithm, we pair the next source to its nearest target.
    /// we move as much mass as we can. we then continue to an arbitrary next source, and repeat.
    ///
    /// this is O(N * M) in (sources, targets) size. for us these are both
    /// the K number of clusters at different layers of the hierarchy.
    ///
    /// if anything, the most expensive part about this is the double BTreeMap
    /// allocation. considering that we do this a few billion times it is
    /// probably worth optimizing into a 0-alloc implementation.
    ///
    /// also, it turns out this algorithm sucks in worst case. like it's just not at all
    /// a reasonable heuristic, even in pathological 1D trivial cases.
    fn greedy(&self, x: &Histogram, y: &Histogram) -> f32 {
        let mut cost = 0.;
        let mut pile = x.normalize();
        let mut sink = y.normalize();
        'cost: while pile.values().any(|&dx| dx > 0.) {
            'pile: for (x, dx) in pile
                .iter_mut()
                .filter(|(_, dx)| **dx > 0.)
                .map(|(&x, dx)| (x, dx))
                .collect::<Vec<_>>()
            {
                match sink
                    .iter_mut()
                    .filter(|(_, dy)| **dy > 0.)
                    .map(|(&y, dy)| ((y, dy), self.distance(&x, &y)))
                    .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                {
                    None => break 'cost,
                    Some(((_, dy), distance)) => {
                        let flow = f32::min(*dx, *dy);
                        *dx -= flow;
                        *dy -= flow;
                        cost += flow * distance;
                        continue 'pile;
                    }
                }
            }
        }
        cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::clustering::histogram::Histogram;

    #[test]
    fn is_turn_emd_zero() {
        let metric = Metric::default();
        let obs = Observation::from(Street::Turn);
        let ref h1 = Histogram::from(obs.clone());
        let ref h2 = Histogram::from(obs.clone());
        assert!(metric.emd(h1, h2) == 0.);
        assert!(metric.emd(h2, h1) == 0.);
    }

    #[test]
    fn is_turn_emd_positive() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        assert!(metric.emd(h1, h2) > 0.);
        assert!(metric.emd(h2, h1) > 0.);
    }

    #[test]
    fn is_turn_emd_symmetric() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        assert!(metric.emd(h1, h2) == metric.emd(h2, h1));
    }
}

// TODO
// encapsulate with some Persist trait
impl Metric {
    /// save profile to disk in a PGCOPY compatible format
    pub fn save(&self, street: Street) {
        log::info!("{:<32}{:<32}", "saving metric", street);
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.metric.pgcopy", street)).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (pair, distance) in self.0.iter() {
            const N_FIELDS: u16 = 2;
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*pair)).unwrap();
            file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
            file.write_f32::<BE>(*distance).unwrap();
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}
