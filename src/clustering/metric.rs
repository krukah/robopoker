use super::equity::Equity;
use super::heuristic::Heuristic;
use super::sinkhorn::Sinkhorn;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::pair::Pair;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use crate::Energy;
use std::collections::BTreeMap;

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default)]
pub struct Metric(BTreeMap<Pair, Energy>);

impl Metric {
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        if x == y {
            0.
        } else {
            self.0
                .get(&Pair::from((x, y)))
                .copied()
                .expect("missing abstraction pair")
        }
    }

    pub fn emd(&self, source: &Histogram, target: &Histogram) -> Energy {
        match source.peek() {
            Abstraction::Learned(_) => Sinkhorn::from((source, target, self)).minimize().cost(),
            Abstraction::Percent(_) => Equity::variation(source, target),
            Abstraction::Preflop(_) => unreachable!("no preflop emd"),
        }
    }
}
impl Metric {
    pub fn done() -> bool {
        Street::all()
            .iter()
            .map(|street| format!("{}.metric.pgcopy", street))
            .any(|file| std::fs::metadata(file).is_ok())
    }
    pub fn load() -> Self {
        log::info!("loading metric");
        let mut map = BTreeMap::default();
        map.extend(Self::from(Street::Pref).0);
        map.extend(Self::from(Street::Flop).0);
        map.extend(Self::from(Street::Turn).0);
        Self(map)
    }
    fn from(street: Street) -> Self {
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let file = File::open(format!("{}.metric.pgcopy", street)).expect("open file");
        let mut buffer = [0u8; 2];
        let mut lookup = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 2 {
                reader.read_u32::<BE>().expect("pair length");
                let pair_i64 = reader.read_i64::<BE>().expect("read pair");
                reader.read_u32::<BE>().expect("distance length");
                let dist_f32 = reader.read_f32::<BE>().expect("read distance");
                let pair = Pair::from(pair_i64);
                lookup.insert(pair, dist_f32);
                continue;
            } else {
                break;
            }
        }
        Self(lookup)
    }
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
}

impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> Energy {
        match (x, y) {
            (Self::X::Learned(_), Self::Y::Learned(_)) => self.lookup(x, y),
            (Self::X::Percent(_), Self::Y::Percent(_)) => Equity.distance(x, y),
            (Self::X::Preflop(_), Self::Y::Preflop(_)) => unreachable!("no preflop distance"),
            _ => unreachable!(),
        }
    }
}

impl From<BTreeMap<Pair, Energy>> for Metric {
    fn from(metric: BTreeMap<Pair, Energy>) -> Self {
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

    #[test]
    fn is_equity_emd_positive() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        let d12 = metric.emd(h1, h2);
        let d21 = metric.emd(h2, h1);
        assert!(d12 > 0.);
        assert!(d21 > 0.);
    }

    #[test]
    fn is_equity_emd_symmetric() {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        let d12 = metric.emd(h1, h2);
        let d21 = metric.emd(h2, h1);
        assert!(d12 == d21);
    }

    #[test]
    fn is_equity_emd_zero() {
        let metric = Metric::default();
        let h = Histogram::from(Observation::from(Street::Turn));
        let d = metric.emd(&h, &h);
        assert!(d == 0.);
    }

    /// this guy is used just to construct arbitrary metric, histogram, histogram tuples
    /// to test transport mechanisms
    fn transport() -> (Metric, Histogram, Histogram) {
        // construct random metric satisfying symmetric semipositivity
        use rand::Rng;
        const MAX_DISTANCE: f32 = 1.0;
        let mut rng = rand::thread_rng();
        let mut metric = BTreeMap::new();
        let p = Histogram::random();
        let q = Histogram::random();
        let support = p.support().chain(q.support()).collect::<Vec<_>>();
        for &x in &support {
            for &y in &support {
                if x > y {
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
    fn is_abstract_emd_positive() {
        let (metric, h1, h2) = transport();
        let d12 = metric.emd(&h1, &h2);
        let d21 = metric.emd(&h2, &h1);
        assert!(d12 > 0., "non positive \n{} \n{}", d12, d21);
        assert!(d21 > 0., "non positive \n{} \n{}", d12, d21);
    }

    #[test]
    fn is_abstract_emd_symmetric() {
        const TOLERANCE: f32 = 0.15;
        let (metric, h1, h2) = transport();
        let d12 = metric.emd(&h1, &h2);
        let d21 = metric.emd(&h2, &h1);
        assert!(
            (d12 - d21).abs() <= TOLERANCE * d12.max(d21),
            "non symmetric \n{} \n{}",
            d12,
            d21
        );
    }

    #[test]
    fn is_abstract_emd_zero() {
        const TOLERANCE: f32 = 0.005;
        let (metric, h1, h2) = transport();
        let d11 = metric.emd(&h1, &h1);
        let d22 = metric.emd(&h2, &h2);
        assert!(d11 <= TOLERANCE);
        assert!(d22 <= TOLERANCE);
    }

    #[test]
    fn persistence() {
        let street = Street::Rive;
        let (save, ..) = transport();
        save.save(street);
        let load = Metric::from(street);
        std::iter::empty()
            .chain(save.0.iter().zip(load.0.iter()))
            .chain(load.0.iter().zip(save.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
        std::fs::remove_file(format!("{}.metric.pgcopy", street)).unwrap();
    }
}
