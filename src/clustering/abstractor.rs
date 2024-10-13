use cards::observation::Observation as Isomorphism;
use cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::progress::Progress;
use std::collections::BTreeMap;

/// this is the output of the clustering module
/// it is a massive table of `Isomorphism` -> `Abstraction`.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
#[derive(Default)]
pub struct Abstractor(BTreeMap<Isomorphism, Abstraction>);

impl Abstractor {
    /// pulls the entire pre-computed abstraction table
    /// into memory. ~50GB.
    pub fn assemble() -> Self {
        let mut map = BTreeMap::default();
        map.extend(Self::load(Street::Turn).0);
        map.extend(Self::load(Street::Flop).0);
        Self(map)
    }

    /// at a given `Street`,
    /// 1. decompose the `Isomorphism` into all of its next-street `Isomorphism`s,
    /// 2. map each of them into an `Abstraction`,
    /// 3. collect the results into a `Histogram`.
    pub fn projection(&self, inner: &Isomorphism) -> Histogram {
        match inner.street() {
            Street::Turn => inner.clone().into(),
            _ => inner
                .outnodes()
                .into_iter()
                .map(|ref outer| self.abstraction(outer))
                .collect::<Vec<Abstraction>>()
                .into(),
        }
    }
    /// lookup the pre-computed abstraction for the outer observation
    pub fn abstraction(&self, outer: &Isomorphism) -> Abstraction {
        self.0
            .get(outer)
            .cloned()
            .expect("precomputed abstraction mapping")
    }
    /// simple insertion.
    /// can we optimize out this clone though?
    pub fn assign(&mut self, abs: &Abstraction, obs: &Isomorphism) {
        self.0.insert(obs.to_owned(), abs.to_owned());
    }

    /// persist the abstraction mapping to disk
    /// write the full abstraction lookup table to disk
    /// 1. Write the PGCOPY header (15 bytes)
    /// 2. Write the flags (4 bytes)
    /// 3. Write the extension (4 bytes)
    /// 4. Write the observation and abstraction pairs
    /// 5. Write the trailer (2 bytes)
    pub fn save(&self, name: String) {
        log::info!("uploading abstraction lookup table {}", name);
        use byteorder::BigEndian;
        use byteorder::WriteBytesExt;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.abstraction.pgcopy", name)).expect("new file");
        let ref mut progress = Progress::new(self.0.len(), 10);
        file.write_all(b"PGCOPY\n\xff\r\n\0").expect("header");
        file.write_u32::<BigEndian>(0).expect("flags");
        file.write_u32::<BigEndian>(0).expect("extension");
        for (observation, abstraction) in self.0.iter() {
            let obs = i64::from(*observation);
            let abs = i64::from(*abstraction);
            file.write_u16::<BigEndian>(2).expect("field count");
            file.write_u32::<BigEndian>(8).expect("8-bytes field");
            file.write_i64::<BigEndian>(obs).expect("observation");
            file.write_u32::<BigEndian>(8).expect("8-bytes field");
            file.write_i64::<BigEndian>(abs).expect("abstraction");
            progress.tick();
        }
        file.write_u16::<BigEndian>(0xFFFF).expect("trailer");
    }
    /// read the full abstraction lookup table from disk
    /// 1. Skip PGCOPY header (15 bytes), flags (4 bytes), and header extension (4 bytes)
    /// 2. Read field count (should be 2)
    /// 3. Read observation length (4 bytes)
    /// 4. Read observation (8 bytes)
    /// 5. Read abstraction length (4 bytes)
    /// 6. Read abstraction (8 bytes)
    /// 7. Insert observation and abstraction into lookup table
    /// 8. Repeat until end of file
    fn load(street: Street) -> Self {
        log::info!("downloading abstraction lookup table {}", street);
        use byteorder::BigEndian;
        use byteorder::ReadBytesExt;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let file = File::open(format!("{}.abstraction.pgcopy", street)).expect("open file");
        let mut buffer = [0u8; 2];
        let mut lookup = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(23)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) != 2 {
                break;
            }
            reader.read_u32::<BigEndian>().expect("observation length");
            let obs = reader.read_i64::<BigEndian>().expect("read observation");
            reader.read_u32::<BigEndian>().expect("abstraction length");
            let abs = reader.read_i64::<BigEndian>().expect("read abstraction");
            let observation = Isomorphism::from(obs);
            let abstraction = Abstraction::from(abs);
            lookup.insert(observation, abstraction);
        }
        Self(lookup)
    }
}

// Test generate projection
#[cfg(test)]
mod tests {
    use super::*;
    use cards::hand::Hand;
    use cards::observation::Observation;

    #[test]
    fn generate_projection() {
        let abstractor = Abstractor::default();
        let inner = Observation::from((Hand::from("As Ad"), Hand::from("Jc 7h Ks Qc")));
        let projection = abstractor.projection(&inner);
        let equity = projection.equity();
        assert!(equity > 0.0);
    }
}