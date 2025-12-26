use crate::cards::*;
use crate::clustering::*;
use crate::gameplay::*;
use rayon::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
/// this is the grand lookup table for all the Isomorphism -> Abstraction
/// mappings. we spend a lot of compute over a lot of hands (all of them!)
/// to precompute this mapping.
pub struct Lookup(BTreeMap<Isomorphism, Abstraction>);

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

impl Lookup {
    /// lookup the pre-computed abstraction for the outer observation
    pub fn lookup(&self, iso: &Isomorphism) -> Abstraction {
        self.0
            .get(iso)
            .cloned()
            .expect("precomputed abstraction in lookup")
    }

    /// generate the entire space of inner layers
    pub fn projections(&self) -> Vec<Histogram> {
        IsomorphismIterator::from(self.street().prev())
            .collect::<Vec<Isomorphism>>()
            .into_par_iter()
            .map(|i| self.future(&i))
            .collect::<Vec<Histogram>>()
    }

    /// distribution over potential next states. this "layer locality" is what
    /// makes imperfect recall hierarchical kmeans nice
    fn future(&self, iso: &Isomorphism) -> Histogram {
        assert!(iso.0.street() != Street::Rive);
        iso.0
            .children()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(Isomorphism::from)
            .map(|i| self.lookup(&i))
            .collect::<Vec<Abstraction>>()
            .into()
    }

    fn street(&self) -> Street {
        self.0.keys().next().expect("non empty").0.street()
    }
}

#[cfg(feature = "database")]
impl crate::save::Schema for Lookup {
    fn name() -> &'static str {
        crate::save::ISOMORPHISM
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        <crate::mccfr::NlheEncoder as crate::save::Schema>::columns()
    }
    fn creates() -> &'static str {
        <crate::mccfr::NlheEncoder as crate::save::Schema>::creates()
    }
    fn indices() -> &'static str {
        <crate::mccfr::NlheEncoder as crate::save::Schema>::indices()
    }
    fn copy() -> &'static str {
        <crate::mccfr::NlheEncoder as crate::save::Schema>::copy()
    }
    fn truncates() -> &'static str {
        <crate::mccfr::NlheEncoder as crate::save::Schema>::truncates()
    }
    fn freeze() -> &'static str {
        <crate::mccfr::NlheEncoder as crate::save::Schema>::freeze()
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl crate::save::Streamable for Lookup {
    type Row = (i64, i16);
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.0
            .into_iter()
            .map(|(iso, abs)| (i64::from(iso), i16::from(abs)))
    }
}

#[cfg(feature = "database")]
impl Lookup {
    pub async fn from_street(client: &tokio_postgres::Client, street: Street) -> Self {
        const SQL: &str =
            const_format::concatcp!("SELECT obs, abs FROM ", crate::save::ISOMORPHISM);
        client
            .query(SQL, &[])
            .await
            .expect("query")
            .into_iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, i16>(1)))
            .filter(|(obs, _)| Street::from(*obs) == street)
            .map(|(obs, abs)| (Isomorphism::from(obs), Abstraction::from(abs)))
            .collect::<BTreeMap<_, _>>()
            .into()
    }
}

impl Lookup {
    /// abstractions for River are calculated once via obs.equity
    /// abstractions for Preflop are equivalent to just enumerating isomorphisms
    pub fn grow(street: Street) -> Self {
        match street {
            Street::Rive => IsomorphismIterator::from(Street::Rive)
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|iso| (iso, Abstraction::from(iso.0.equity())))
                .collect::<BTreeMap<_, _>>()
                .into(),
            Street::Pref => IsomorphismIterator::from(Street::Pref)
                .enumerate()
                .map(|(k, iso)| (iso, Abstraction::from((Street::Pref, k))))
                .collect::<BTreeMap<_, _>>()
                .into(),
            Street::Flop | Street::Turn => panic!("lookup must be learned via layer for {street}"),
        }
    }
}

#[cfg(feature = "disk")]
#[allow(deprecated)]
impl crate::save::Disk for Lookup {
    fn name() -> &'static str {
        crate::save::ISOMORPHISM
    }
    fn grow(street: Street) -> Self {
        Lookup::grow(street)
    }
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
        log::info!("{:<32}{:<32}", "loading     lookup", path.display());
        use byteorder::BE;
        use byteorder::ReadBytesExt;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let ref file = File::open(path).expect(&format!("open {}", path.display()));
        let mut lookup = BTreeMap::new();
        let mut reader = BufReader::new(file);
        let ref mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(buffer).is_ok() {
            match u16::from_be_bytes(buffer.clone()) {
                2 => {
                    assert!(8 == reader.read_u32::<BE>().expect("observation length"));
                    let iso = reader.read_i64::<BE>().expect("read observation");
                    assert!(2 == reader.read_u32::<BE>().expect("abstraction length"));
                    let abs = reader.read_i16::<BE>().expect("read abstraction");
                    let observation = Isomorphism::from(iso);
                    let abstraction = Abstraction::from(abs);
                    lookup.insert(observation, abstraction);
                }
                0xFFFF => break,
                n => panic!("unexpected number of fields: {}", n),
            }
        }
        Self(lookup)
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
        log::info!("{:<32}{:<32}", "saving      lookup", path.display());
        file.write_all(Self::header()).expect("header");
        for (Isomorphism(obs), abs) in self.0.iter() {
            file.write_u16::<BE>(N_FIELDS).unwrap();
            file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
            file.write_i64::<BE>(i64::from(*obs)).unwrap();
            file.write_u32::<BE>(size_of::<i16>() as u32).unwrap();
            file.write_i16::<BE>(i16::from(*abs)).unwrap();
        }
        file.write_u16::<BE>(Self::footer()).expect("trailer");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    #[cfg(feature = "disk")]
    fn persistence() {
        use crate::cards::*;
        use crate::clustering::*;
        use crate::save::*;
        let street = Street::Pref;
        let lookup = Lookup::grow(street);
        lookup.save();
        let loaded = Lookup::load(street);
        std::iter::empty()
            .chain(lookup.0.iter().zip(loaded.0.iter()))
            .chain(loaded.0.iter().zip(lookup.0.iter()))
            .all(|((s1, l1), (s2, l2))| s1 == s2 && l1 == l2);
    }
}
