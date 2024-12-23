use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::Save;

use std::collections::BTreeMap;

pub struct Decomp(BTreeMap<Abstraction, Histogram>);
impl Decomp {
    const SUFFIX: &'static str = ".transition.pgcopy";
}
impl Save for Decomp {
    fn make(street: Street) -> Self {
        unreachable!("you have no business making transition table from scratch {street}")
    }
    fn done(street: Street) -> bool {
        std::fs::metadata(format!("{}{}", street, Self::SUFFIX)).is_ok()
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
        let mut transitions = BTreeMap::new();
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(&mut buffer).is_ok() {
            if u16::from_be_bytes(buffer) == 3 {
                reader.read_u32::<BE>().expect("from abstraction");
                let from_i64 = reader.read_i64::<BE>().expect("read from abstraction");
                reader.read_u32::<BE>().expect("into abstraction");
                let into_i64 = reader.read_i64::<BE>().expect("read into abstraction");
                reader.read_u32::<BE>().expect("weight");
                let weight = reader.read_f32::<BE>().expect("read weight");
                let from_abs = Abstraction::from(from_i64);
                let into_abs = Abstraction::from(into_i64);
                let mass = (street.next().n_observations() / street.n_observations()) as f32;
                transitions
                    .entry(from_abs)
                    .or_insert_with(Histogram::default)
                    .set(into_abs, (weight * mass) as usize);
                continue;
            } else {
                break;
            }
        }
        Self(transitions)
    }
    fn save(&self) {
        let street = self
            .0
            .keys()
            .next()
            .copied()
            .unwrap_or_else(|| Abstraction::from(0.))
            .street();
        log::info!("{:<32}{:<32}", "saving transition", street);
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        let ref mut file = File::create(format!("{}{}", street, Self::SUFFIX)).expect("touch");
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (current_abs, histogram) in self.0.iter() {
            for next_abs in histogram.support() {
                const N_FIELDS: u16 = 3;
                file.write_u16::<BE>(N_FIELDS).unwrap();
                file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
                file.write_i64::<BE>(i64::from(*current_abs)).unwrap();
                file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
                file.write_i64::<BE>(i64::from(*next_abs)).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(histogram.density(next_abs)).unwrap();
            }
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}
impl From<BTreeMap<Abstraction, Histogram>> for Decomp {
    fn from(map: BTreeMap<Abstraction, Histogram>) -> Self {
        Self(map)
    }
}
