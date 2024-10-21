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
            (Self::X::Random(_), Self::Y::Random(_)) => self.lookup(x, y),
            (Self::X::Equity(a), Self::Y::Equity(b)) => Equity.distance(a, b),
            _ => unreachable!("only equity distance for river abstractions"),
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
        self.greedy(x, y)
    }
}

impl Metric {
    pub fn emd(&self, source: &Histogram, target: &Histogram) -> f32 {
        match source.peek() {
            Abstraction::Random(_) => self.greedy(source, target),
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
    fn greedy(&self, x: &Histogram, y: &Histogram) -> f32 {
        let mut cost = 0.;
        let mut sources = x.normalized();
        let mut targets = y.normalized();
        'cost: while sources.values().any(|&v| v > 0.) {
            'flow: for (x, pile) in sources
                .iter()
                // .cycle()
                .filter(|(_, &pile)| pile > 0.)
                .map(|(&x, &pile)| (x, pile))
                .collect::<Vec<_>>()
            {
                let nearest = targets
                    .iter()
                    .filter(|(_, &hole)| hole > 0.)
                    .map(|(&y, &hole)| ((y, hole), self.distance(&x, &y)))
                    .min_by(|(_, y1), (_, y2)| y1.partial_cmp(y2).unwrap());
                match nearest {
                    None => break 'cost,
                    Some(((y, hole), distance)) => {
                        if pile > hole {
                            let earth = hole;
                            cost += distance * earth;
                            *sources.get_mut(&x).unwrap() -= earth;
                            *targets.get_mut(&y).unwrap() = 0.;
                            continue 'flow;
                        } else {
                            let earth = pile;
                            cost += distance * earth;
                            *sources.get_mut(&x).unwrap() = 0.;
                            *targets.get_mut(&y).unwrap() -= earth;
                            continue 'flow;
                        }
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
        log::info!("{:<32}{:<32}", "uploading abstraction metric", street);
        use byteorder::BigEndian;
        use byteorder::WriteBytesExt;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.metric.pgcopy", street)).expect("new file");
        file.write_all(b"PGCOPY\n\xff\r\n\0").expect("header");
        file.write_u32::<BigEndian>(0).expect("flags");
        file.write_u32::<BigEndian>(0).expect("extension");
        for (pair, distance) in self.0.iter() {
            file.write_u16::<BigEndian>(2).expect("field count");
            file.write_u32::<BigEndian>(8).expect("8-bytes field");
            file.write_i64::<BigEndian>(i64::from(*pair)).expect("pair");
            file.write_u32::<BigEndian>(4).expect("4-bytes field");
            file.write_f32::<BigEndian>(*distance).expect("distance");
        }
        file.write_u16::<BigEndian>(0xFFFF).expect("trailer");
    }
}
