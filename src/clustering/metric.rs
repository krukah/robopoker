use super::equity::Equity;
use super::sinkhorn::Sinkhorn;
use crate::cards::street::Street;
use crate::clustering::histogram::Histogram;
use crate::clustering::pair::Pair;
use crate::gameplay::abstraction::Abstraction;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use crate::Energy;
use std::collections::BTreeMap;

/// Distance metric for kmeans clustering.
/// encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Default, Clone)]
pub struct Metric(BTreeMap<Pair, Energy>);

impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> Energy {
        match (x, y) {
            _ if x == y => 0.,
            (Self::X::Learned(_), Self::Y::Learned(_)) => self.lookup(x, y),
            (Self::X::Percent(_), Self::Y::Percent(_)) => Equity.distance(x, y),
            (Self::X::Preflop(_), Self::Y::Preflop(_)) => unreachable!("no preflop distance"),
            _ => unreachable!(),
        }
    }
}

impl Metric {
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        self.0
            .get(&Pair::from((x, y)))
            .copied()
            .expect("missing abstraction pair")
    }

    pub fn emd(&self, source: &Histogram, target: &Histogram) -> Energy {
        match source.peek() {
            Abstraction::Learned(_) => Sinkhorn::from((source, target, self)).minimize().cost(),
            Abstraction::Percent(_) => Equity::variation(source, target),
            Abstraction::Preflop(_) => unreachable!("no preflop emd"),
        }
    }

    /// we're assuming tht the street is being generated AFTER the learned kmeans
    /// cluster distance calculation. so we should have (Street::K() choose 2)
    /// entreis in our abstraction pair lookup table.
    /// if this is off by just a few then it probably means a bunch of collisions
    /// maybe i should determinsitcally seed kmeans process, could be cool for reproducability too
    ///
    /// TODO
    ///
    /// determine street dynamiccaly by checking for existence of XOR'ed abstraction pairs using
    /// Abstraction::From(Street, Index)
    ///
    /// it's also not great that we are FORCED to have different number of abstractions
    /// clusters K means for each street to avoid nC2 collisions !!
    /// we should either just store Street as Self.1 or determine from XOR hits what street we're on
    /// whichever solution should work with test case so we don't have to remove test case
    /// to not overwrite existing metric. we like overwriting river.metric bc it can be empty
    fn street(&self) -> Street {
        fn choose_2(k: usize) -> usize {
            k * (k.saturating_sub(1)) / 2
        }
        match self.0.len() {
            n if n == choose_2(Street::Rive.k()) => Street::Rive,
            n if n == choose_2(Street::Turn.k()) => Street::Turn,
            n if n == choose_2(Street::Flop.k()) => Street::Flop,
            n if n == choose_2(Street::Pref.k()) => Street::Pref,
            _ => Street::Rive, // assertion of no-collisions is convenient for tests
        }
    }

    fn name() -> String {
        "metric".to_string()
    }
}

impl crate::save::upload::Table for Metric {
    fn name() -> String {
        Self::name()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::FLOAT4,
        ]
    }
    fn sources() -> Vec<std::path::PathBuf> {
        use crate::save::disk::Disk;
        Street::all()
            .iter()
            .rev()
            .copied()
            .map(Self::path)
            .collect()
    }
    fn copy() -> String {
        "COPY metric (
            xor,
            dx
        )
        FROM STDIN BINARY
        "
        .to_string()
    }
    fn creates() -> String {
        "
        CREATE TABLE IF NOT EXISTS metric (
            xor        BIGINT,
            dx         REAL
        );
        "
        .to_string()
    }
    fn indices() -> String {
        "
        INSERT INTO metric (
            xor,
            dx
        ) VALUES (
            0,
            0
        );
        CREATE INDEX IF NOT EXISTS idx_metric_xor ON metric (xor);
        CREATE INDEX IF NOT EXISTS idx_metric_dx  ON metric (dx);
        "
        .to_string()
    }
}

impl crate::save::disk::Disk for Metric {
    fn load(street: Street) -> Self {
        let ref path = Self::path(street);
        log::info!("{:<32}{:<32}", "loading     metric", path.display());
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let ref file = File::open(path).expect(&format!("open {}", path.display()));
        let mut metric = BTreeMap::new();
        let mut reader = BufReader::new(file);
        let ref mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(buffer).is_ok() {
            match u16::from_be_bytes(buffer.clone()) {
                2 => {
                    reader.read_u32::<BE>().expect("pair length");
                    let pair = reader.read_i64::<BE>().expect("read pair");
                    reader.read_u32::<BE>().expect("distance length");
                    let dist = reader.read_f32::<BE>().expect("read distance");
                    metric.insert(Pair::from(pair), dist);
                    continue;
                }
                0xFFFF => break,
                n => panic!("unexpected number of fields: {}", n),
            }
        }
        Self(metric)
    }
    fn save(&self) {
        const N_FIELDS: u16 = 2;
        let street = self.street();
        let ref path = Self::path(street);
        let ref mut file = File::create(path).expect(&format!("touch {}", path.display()));
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        log::info!("{:<32}{:<32}", "saving      metric", path.display());
        file.write_all(Self::header()).expect("header");
        for (pair, distance) in self.0.iter() {
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*pair)).unwrap();
            file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
            file.write_f32::<BE>(*distance).unwrap();
        }
        file.write_u16::<BE>(Self::footer()).expect("trailer");
    }
    fn grow(_: Street) -> Self {
        unreachable!("metric must be learned from kmeans clustering")
    }

    fn name() -> String {
        Self::name()
    }
}

impl From<BTreeMap<Pair, Energy>> for Metric {
    fn from(metric: BTreeMap<Pair, Energy>) -> Self {
        let max = metric.values().copied().fold(f32::MIN_POSITIVE, f32::max);
        Self(
            metric
                .into_iter()
                .map(|(index, distance)| (index, distance / max))
                .collect(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::street::Street;
    use crate::clustering::emd::EMD;
    use crate::save::disk::Disk;
    use crate::Arbitrary;

    #[test]
    #[ignore]
    fn persistence() {
        let street = Street::Rive;
        let emd = EMD::random();
        let save = emd.metric();
        save.save();
        let load = Metric::load(street);
        std::iter::empty()
            .chain(save.0.iter().zip(load.0.iter()))
            .chain(load.0.iter().zip(save.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
    }
}
