use crate::cards::isomorphism::Isomorphism;
use crate::cards::isomorphisms::IsomorphismIterator;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::Save;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Lookup(BTreeMap<Isomorphism, Abstraction>);

impl Lookup {
    const SUFFIX: &'static str = ".encoder.pgcopy";
    /// lookup the pre-computed abstraction for the outer observation
    pub fn lookup(&self, obs: &Observation) -> Abstraction {
        self.0
            .get(&Isomorphism::from(*obs))
            .cloned()
            .expect(&format!("precomputed abstraction missing for {obs}"))
    }
    /// generate the entire space of inner layers
    pub fn projections(&self) -> Vec<Histogram> {
        use rayon::iter::IntoParallelIterator;
        IsomorphismIterator::from(self.street().prev())
            .collect::<Vec<Isomorphism>>()
            .into_par_iter()
            .map(|inner| self.future(&inner))
            .collect::<Vec<Histogram>>()
    }
    /// distribution over potential next states. this "layer locality" is what
    /// makes imperfect recall hierarchical kmeans nice
    fn future(&self, iso: &Isomorphism) -> Histogram {
        assert!(iso.0.street() != Street::Rive);
        iso.0
            .children()
            .map(|o| self.lookup(&o))
            .collect::<Vec<Abstraction>>()
            .into()
    }
    fn street(&self) -> Street {
        self.0.keys().next().expect("non empty").0.street()
    }
}

impl Save for Lookup {
    fn done(street: Street) -> bool {
        std::fs::metadata(format!("{}{}", street, Self::SUFFIX)).is_ok()
    }
    fn make(street: Street) -> Self {
        match street {
            Street::Rive => IsomorphismIterator::from(Street::Rive)
                .map(|iso| (iso, Abstraction::from(iso.0.equity())))
                .collect::<BTreeMap<_, _>>()
                .into(),
            Street::Pref => IsomorphismIterator::from(Street::Pref)
                .enumerate()
                .map(|(k, iso)| (iso, Abstraction::from((Street::Pref, k))))
                .collect::<BTreeMap<_, _>>()
                .into(),
            _ => panic!("lookup must be learned via layer for {street}"),
        }
    }
    fn load(street: Street) -> Self {
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let file = File::open(format!("{}{}", street, Self::SUFFIX)).expect("open file");
        let mut reader = BufReader::new(file);
        let mut lookup = BTreeMap::new();
        let mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 2 {
                reader.read_u32::<BE>().expect("observation length");
                let iso = reader.read_i64::<BE>().expect("read observation");
                reader.read_u32::<BE>().expect("abstraction length");
                let abs = reader.read_i64::<BE>().expect("read abstraction");
                let observation = Isomorphism::from(iso);
                let abstraction = Abstraction::from(abs);
                lookup.insert(observation, abstraction);
                continue;
            } else {
                break;
            }
        }
        Self(lookup)
    }
    fn save(&self) {
        let street = self.street();
        log::info!("{:<32}{:<32}", "saving encoding", street);
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}{}", street, Self::SUFFIX)).expect("touch");
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
    use crate::Save;

    #[test]
    fn persistence() {
        let street = Street::Pref;
        let lookup = Lookup::make(street);
        lookup.save();
        let loaded = Lookup::load(street);
        std::iter::empty()
            .chain(lookup.0.iter().zip(loaded.0.iter()))
            .chain(loaded.0.iter().zip(lookup.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
    }
}

impl From<Lookup> for BTreeMap<Isomorphism, Abstraction> {
    fn from(lookup: Lookup) -> BTreeMap<Isomorphism, Abstraction> {
        lookup.0
    }
}
impl From<BTreeMap<Isomorphism, Abstraction>> for Lookup {
    fn from(map: BTreeMap<Isomorphism, Abstraction>) -> Self {
        Self(map)
    }
}
