use super::equity::Equity;
use super::sinkhorn::Sinkhorn;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::xor::Pair;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use crate::Distance;
use std::collections::BTreeMap;

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default)]
pub struct Metric(BTreeMap<Pair, Distance>);

impl Metric {
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> Distance {
        if x == y {
            0.
        } else {
            self.0
                .get(&Pair::from((x, y)))
                .copied()
                .expect("missing abstraction pair")
        }
    }

    /// save profile to disk in a PGCOPY compatible format
    pub fn save(&self, street: Street) {
        println!("{:<32}{:<32}", "saving metric", street);
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

    pub fn emd(&self, source: &Histogram, target: &Histogram) -> Distance {
        match source.peek() {
            Abstraction::Learned(_) => Sinkhorn::from((source, target, self)).minimize().cost(),
            Abstraction::Percent(_) => Equity::variation(source, target),
            Abstraction::Preflop(_) => unreachable!("no preflop emd"),
        }
    }
}

impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> Distance {
        match (x, y) {
            (Self::X::Learned(_), Self::Y::Learned(_)) => self.lookup(x, y),
            (Self::X::Percent(_), Self::Y::Percent(_)) => Equity.distance(x, y),
            (Self::X::Preflop(_), Self::Y::Preflop(_)) => unreachable!("no preflop distance"),
            _ => unreachable!(),
        }
    }
}

impl From<BTreeMap<Pair, Distance>> for Metric {
    fn from(metric: BTreeMap<Pair, Distance>) -> Self {
        Self(metric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::clustering::histogram::Histogram;
    use crate::Arbitrary;
    use rand::thread_rng;
    use rand::Rng;

    fn transportation() -> (Metric, Histogram, Histogram) {
        // construct random metric satisfying symmetric semipositivity
        const MAX_DISTANCE: f32 = 1.0;
        let mut rng = thread_rng();
        let mut metric = BTreeMap::new();
        let p = Histogram::random();
        let q = Histogram::random();
        for x in p.support() {
            for y in q.support() {
                if x != y {
                    let dist = rng.gen_range(0.0..MAX_DISTANCE);
                    let pair = Pair::from((x, y));
                    metric.insert(pair, dist);
                }
            }
        }
        let m = Metric(metric);
        (m, p, q)
    }

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

    #[test]
    fn is_transport_emd_zero() {
        let (metric, h1, h2) = transportation();
        assert!(metric.emd(&h1, &h1) == 0.);
        assert!(metric.emd(&h2, &h2) == 0.);
    }

    #[test]
    fn is_transport_emd_positive() {
        let (metric, h1, h2) = transportation();
        assert!(metric.emd(&h1, &h2) > 0.);
        assert!(metric.emd(&h2, &h1) > 0.);
    }

    #[test]
    fn is_transport_emd_symmetric() {
        let (metric, h1, h2) = transportation();
        assert!(metric.emd(&h1, &h2) == metric.emd(&h2, &h1));
    }
}
