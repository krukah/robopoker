use super::*;
use crate::cards::*;
use crate::gameplay::*;
use crate::transport::*;
use crate::*;

/// Distance metric for kmeans clustering.
/// Encapsulates distance between `Abstraction`s of the "previous" hierarchy,
/// as well as: distance between `Histogram`s of the "current" hierarchy.
#[derive(Clone, Copy)]
pub enum Metric {
    Pref(DistPref),
    Flop(DistFlop),
    Turn(DistTurn),
    Rive,
}

impl Default for Metric {
    fn default() -> Self {
        Metric::Pref(Distances::new(Street::Pref))
    }
}

impl Measure for Metric {
    type X = Abstraction;
    type Y = Abstraction;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> Energy {
        if x == y {
            0.
        } else {
            match (x.street(), y.street()) {
                (Street::Flop, Street::Flop) | (Street::Turn, Street::Turn) => self.lookup(x, y),
                (Street::Rive, Street::Rive) => Equity.distance(x, y),
                (Street::Pref, Street::Pref) => unreachable!("no preflop distance"),
                _ => unreachable!("mismatched streets"),
            }
        }
    }
}

impl Metric {
    pub const fn new(street: Street) -> Self {
        match street {
            Street::Pref => Metric::Pref(Distances::new(street)),
            Street::Flop => Metric::Flop(Distances::new(street)),
            Street::Turn => Metric::Turn(Distances::new(street)),
            Street::Rive => Metric::Rive,
        }
    }
    pub fn street(&self) -> Street {
        match self {
            Metric::Pref(_) => Street::Pref,
            Metric::Flop(_) => Street::Flop,
            Metric::Turn(_) => Street::Turn,
            Metric::Rive => Street::Rive,
        }
    }
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        let pair = Pair::from((x, y));
        match self {
            Metric::Pref(_) => unreachable!("no metric over Histogram<Preflop>"),
            Metric::Flop(d) => d.get(pair),
            Metric::Turn(d) => d.get(pair),
            Metric::Rive => unreachable!("no metric over Histogram<River>"),
        }
    }
    pub fn set(&mut self, pair: Pair, value: Energy) {
        match self {
            Metric::Pref(d) => d.set(pair, value),
            Metric::Flop(d) => d.set(pair, value),
            Metric::Turn(d) => d.set(pair, value),
            Metric::Rive => unreachable!("no metric over Histogram<River>"),
        }
    }
    pub fn emd(&self, source: &Histogram, target: &Histogram) -> Energy {
        match source.peek().street() {
            Street::Flop | Street::Turn => Sinkhorn::from((source, target, self)).minimize().cost(),
            Street::Rive => Equity::variation(source, target),
            Street::Pref => unreachable!("no preflop emd"),
        }
    }
    /// Normalize all distances by the maximum value.
    pub fn normalize(&mut self) {
        match self {
            Metric::Pref(d) => d.normalize(),
            Metric::Flop(d) => d.normalize(),
            Metric::Turn(d) => d.normalize(),
            Metric::Rive => {}
        }
    }
}

impl From<std::collections::BTreeMap<Pair, Energy>> for Metric {
    fn from(map: std::collections::BTreeMap<Pair, Energy>) -> Self {
        let max = map.values().copied().fold(f32::MIN_POSITIVE, f32::max);
        let mut metric = map
            .keys()
            .next()
            .map(|p| p.street())
            .map(Metric::new)
            .expect("map is empty");
        for (pair, distance) in map {
            metric.set(pair, distance / max);
        }
        metric
    }
}

impl IntoIterator for Metric {
    type Item = (i32, Energy);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + Send>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Metric::Pref(d) => d.into_iter(),
            Metric::Flop(d) => d.into_iter(),
            Metric::Turn(d) => d.into_iter(),
            Metric::Rive => unreachable!(),
        }
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl crate::save::Streamable for Metric {
    type Row = (i32, f32);
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.into_iter()
    }
}
#[cfg(feature = "database")]
impl Metric {
    pub async fn from_street(client: &tokio_postgres::Client, street: Street) -> Self {
        const SQL: &str = const_format::concatcp!("SELECT tri, dx FROM ", crate::save::METRIC);
        let mut keys = std::collections::HashSet::new();
        for ref x in Abstraction::all(street) {
            for ref y in Abstraction::all(street) {
                if x < y {
                    keys.insert(i32::from(Pair::from((x, y))));
                }
            }
        }
        let mut metric = Metric::new(street);
        client
            .query(SQL, &[])
            .await
            .expect("query")
            .into_iter()
            .map(|row| (row.get::<_, i32>(0), row.get::<_, f32>(1)))
            .filter(|(tri, _)| keys.contains(tri))
            .map(|(tri, dx)| (Pair::from(tri), dx))
            .for_each(|(pair, dx)| metric.set(pair, dx));
        metric
    }
}

#[cfg(feature = "database")]
impl crate::save::Schema for Metric {
    fn name() -> &'static str {
        crate::save::METRIC
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT4,
            tokio_postgres::types::Type::FLOAT4,
        ]
    }
    fn copy() -> &'static str {
        const_format::concatcp!("COPY ", crate::save::METRIC, " (tri, dx) FROM STDIN BINARY")
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            crate::save::METRIC,
            " (
                tri        INTEGER,
                dx         REAL
            );
            CREATE OR REPLACE FUNCTION get_pair_tri(abs1 SMALLINT, abs2 SMALLINT) RETURNS INTEGER AS
            $$ DECLARE
                street INTEGER;
                i1 INTEGER;
                i2 INTEGER;
                lo INTEGER;
                hi INTEGER;
            BEGIN
                street := (abs1 >> 8) & 255;
                i1 := abs1 & 255;
                i2 := abs2 & 255;
                IF i1 < i2 THEN
                    lo := i1;
                    hi := i2;
                ELSE
                    lo := i2;
                    hi := i1;
                END IF;
                IF hi = 0 THEN
                    RETURN (street << 30);
                ELSE
                    RETURN (street << 30) | (hi * (hi - 1) / 2 + lo);
                END IF;
            END;
            $$ LANGUAGE plpgsql;"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "INSERT INTO ",
            crate::save::METRIC,
            " (tri, dx) VALUES (0, 0);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::METRIC,
            "_tri ON ",
            crate::save::METRIC,
            " (tri);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::METRIC,
            "_dx  ON ",
            crate::save::METRIC,
            " (dx);"
        )
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", crate::save::METRIC, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            crate::save::METRIC,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            crate::save::METRIC,
            " SET (autovacuum_enabled = false);"
        )
    }
}

#[cfg(feature = "disk")]
#[allow(deprecated)]
impl crate::save::Disk for Metric {
    fn sources() -> Vec<std::path::PathBuf> {
        Street::all()
            .iter()
            .rev()
            .copied()
            .map(Self::path)
            .collect()
    }
    fn load(street: Street) -> Self {
        let ref path = Self::path(street);
        log::info!("{:<32}{:<32}", "loading     metric", path.display());
        use byteorder::BE;
        use byteorder::ReadBytesExt;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let ref file = File::open(path).expect(&format!("open {}", path.display()));
        let mut metric = Metric::new(street);
        let mut reader = BufReader::new(file);
        let ref mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(buffer).is_ok() {
            match u16::from_be_bytes(buffer.clone()) {
                2 => {
                    reader.read_u32::<BE>().expect("pair length");
                    let pair = reader.read_i32::<BE>().expect("read pair");
                    reader.read_u32::<BE>().expect("distance length");
                    let dist = reader.read_f32::<BE>().expect("read distance");
                    metric.set(Pair::from(pair), dist);
                    continue;
                }
                0xFFFF => break,
                n => panic!("unexpected number of fields: {}", n),
            }
        }
        metric
    }
    fn save(&self) {
        const N_FIELDS: u16 = 2;
        let street = self.street();
        let ref path = Self::path(street);
        let ref mut file = File::create(path).expect(&format!("touch {}", path.display()));
        use byteorder::BE;
        use byteorder::WriteBytesExt;
        use std::fs::File;
        use std::io::Write;
        log::info!("{:<32}{:<32}", "saving      metric", path.display());
        file.write_all(Self::header()).expect("header");
        for (tri, distance) in (*self).into_iter() {
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i32>() as u32).unwrap();
            file.write_i32::<BE>(tri).unwrap();
            file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
            file.write_f32::<BE>(distance).unwrap();
        }
        file.write_u16::<BE>(Self::footer()).expect("trailer");
    }
    fn grow(_: Street) -> Self {
        unreachable!("metric must be learned from kmeans clustering")
    }
    fn name() -> &'static str {
        crate::save::METRIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pair_triangular_roundtrip() {
        for k in [10, 50, 100] {
            for i in 0..k {
                for j in (i + 1)..k {
                    let pair = Pair::new(Street::Flop, i, j);
                    let (ri, rj) = pair.indices();
                    assert_eq!((i, j), (ri, rj), "roundtrip failed for ({}, {})", i, j);
                }
            }
        }
    }
    #[test]
    fn pair_abstractions_roundtrip() {
        let street = Street::Flop;
        let a = Abstraction::from((street, 5));
        let b = Abstraction::from((street, 10));
        let pair = Pair::from((&a, &b));
        let (ra, rb) = pair.abstractions();
        assert_eq!(a, ra);
        assert_eq!(b, rb);
    }
    #[test]
    #[ignore]
    #[cfg(feature = "disk")]
    fn persistence() {
        use crate::save::*;
        let street = Street::Rive;
        let emd = EMD::random();
        let save = emd.metric();
        save.save();
        let load = Metric::load(street);
        save.into_iter()
            .zip(load.into_iter())
            .all(|((p1, d1), (p2, d2))| p1 == p2 && d1 == d2);
    }
}
