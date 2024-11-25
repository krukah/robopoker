use super::layer::Layer;
use crate::cards::hole::Hole;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use std::collections::BTreeMap;

/// this is the output of the clustering module
/// it is a massive table of `Equivalence` -> `Abstraction`.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
#[derive(Default)]
pub struct Encoder(BTreeMap<Isomorphism, Abstraction>);

impl Encoder {
    /// only run this once.
    pub fn learn() {
        if Self::done() {
            log::info!("skipping abstraction");
        } else {
            log::info!("learning abstraction");
            Layer::outer()
                .inner() // turn
                .inner() // flop
                .inner(); // preflop
        }
    }
    /// simple insertion.
    /// can we optimize out this clone though? maybe for key but not for value
    pub fn assign(&mut self, abs: &Abstraction, iso: &Isomorphism) {
        self.0.insert(iso.clone(), abs.clone());
    }
    /// lookup the pre-computed abstraction for the outer observation
    /// for preflop, we lookup the Hole cards, up to isomorphism
    /// for river, we compute the equity on the fly. could use MC sampling to speed up
    /// for turn and flop, we lookup the pre-computed abstraction that we woked so hard for in ::clustering
    pub fn abstraction(&self, outer: &Isomorphism) -> Abstraction {
        let observation = outer.0;
        match observation.street() {
            Street::Pref => Abstraction::from(Hole::from(observation)),
            Street::Rive => Abstraction::from(observation.equity()),
            Street::Flop | Street::Turn => self
                .0
                .get(outer)
                .cloned()
                .expect("precomputed abstraction mapping for Turn/Flop"),
        }
    }
    /// at a given `Street`,
    /// 1. decompose the `Isomorphism` into all of its next-street `Isomorphism`s,
    /// 2. map each of them into an `Abstraction`,
    /// 3. collect the results into a `Histogram`.
    pub fn projection(&self, inner: &Isomorphism) -> Histogram {
        let observation = inner.0;
        match observation.street() {
            Street::Rive => unreachable!("never project outermost abstraction layer"),
            Street::Turn => Histogram::from(observation),
            Street::Pref | Street::Flop => Histogram::from(
                observation
                    .children()
                    .map(|outer| Isomorphism::from(outer)) // isomorphism translation
                    .map(|outer| self.abstraction(&outer)) // abstraction lookup
                    .collect::<Vec<Abstraction>>(), // histogram collection
            ),
        }
    }
}

/// persistence methods
impl Encoder {
    pub fn done() -> bool {
        Street::all()
            .iter()
            .map(|street| format!("{}.abstraction.pgcopy", street))
            .any(|file| std::fs::metadata(file).is_ok())
    }
    pub fn load() -> Self {
        log::info!("loading encoder");
        let mut map = BTreeMap::default();
        map.extend(Self::from(Street::Flop).0);
        map.extend(Self::from(Street::Turn).0);
        Self(map)
    }
    pub fn from(street: Street) -> Self {
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let file = File::open(format!("{}.abstraction.pgcopy", street)).expect("open file");
        let mut buffer = [0u8; 2];
        let mut lookup = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 2 {
                reader.read_u32::<BE>().expect("observation length");
                let iso_i64 = reader.read_i64::<BE>().expect("read observation");
                reader.read_u32::<BE>().expect("abstraction length");
                let abs_i64 = reader.read_i64::<BE>().expect("read abstraction");
                let observation = Isomorphism::from(iso_i64);
                let abstraction = Abstraction::from(abs_i64);
                lookup.insert(observation, abstraction);
                continue;
            } else {
                break;
            }
        }
        Self(lookup)
    }
    pub fn save(&self, street: Street) {
        log::info!("{:<32}{:<32}", "saving encoding", street);
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}.abstraction.pgcopy", street)).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (Isomorphism(obs), abs) in self.0.iter() {
            const N_FIELDS: u16 = 2;
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*obs)).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*abs)).unwrap();
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;

    #[test]
    fn persistence() {
        let street = Street::Rive;
        let file = format!("{}.abstraction.pgcopy", street);
        let save = Encoder(
            (0..100)
                .map(|_| Observation::from(street))
                .map(|o| Isomorphism::from(o))
                .map(|o| (o, Abstraction::random()))
                .collect(),
        );
        save.save(street);
        let load = Encoder::from(street);
        std::iter::empty()
            .chain(save.0.iter().zip(load.0.iter()))
            .chain(load.0.iter().zip(save.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
        std::fs::remove_file(format!("{}", file)).unwrap();
    }
}
