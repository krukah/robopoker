use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::Save;
use std::collections::BTreeMap;

pub struct Decomp(BTreeMap<Abstraction, Histogram>);

impl From<BTreeMap<Abstraction, Histogram>> for Decomp {
    fn from(map: BTreeMap<Abstraction, Histogram>) -> Self {
        Self(map)
    }
}

impl Save for Decomp {
    fn name() -> &'static str {
        "pgcopy/transitions"
    }
    fn make(street: Street) -> Self {
        unreachable!("you have no business making transition table from scratch {street}")
    }
    fn load(street: Street) -> Self {
        log::info!("{:<32}{:<32}", "loading     transitions", street);
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let ref mass = street.n_children() as f32;
        let ref path = Self::path(street);
        let ref file = File::open(path).expect(&format!("open {}", path));
        let mut decomp = BTreeMap::new();
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(19)).expect("seek past header");

        let ref mut buffer = [0u8; 2];
        while reader.read_exact(buffer).is_ok() {
            match u16::from_be_bytes(buffer.clone()) {
                3 => {
                    reader.read_u32::<BE>().expect("from abstraction");
                    let from = reader.read_i64::<BE>().expect("read from abstraction");
                    reader.read_u32::<BE>().expect("into abstraction");
                    let into = reader.read_i64::<BE>().expect("read into abstraction");
                    reader.read_u32::<BE>().expect("weight");
                    let weight = reader.read_f32::<BE>().expect("read weight");
                    decomp
                        .entry(Abstraction::from(from))
                        .or_insert_with(Histogram::default)
                        .set(Abstraction::from(into), (weight * mass) as usize);
                    continue;
                }
                n => panic!("unexpected number of fields: {}", n),
            }
        }
        Self(decomp)
    }
    fn save(&self) {
        const N_FIELDS: u16 = 3;
        let street = self
            .0
            .keys()
            .next()
            .copied()
            .unwrap_or_else(|| Abstraction::from(0f32)) // coerce to River equity Abstraction if empty
            .street();
        let ref path = Self::path(street);
        let ref mut file = File::create(path).expect(&format!("touch {}", path));
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        log::info!("{:<32}{:<32}", "saving      transition", path);
        file.write_all(b"PGCOPY\n\xFF\r\n\0").expect("header");
        file.write_u32::<BE>(0).expect("flags");
        file.write_u32::<BE>(0).expect("extension");
        for (from, histogram) in self.0.iter() {
            for into in histogram.support() {
                file.write_u16::<BE>(N_FIELDS).unwrap();
                file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
                file.write_i64::<BE>(i64::from(*from)).unwrap();
                file.write_u32::<BE>(size_of::<i64>() as u32).unwrap();
                file.write_i64::<BE>(i64::from(*into)).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(histogram.density(into)).unwrap();
            }
        }
        file.write_u16::<BE>(0xFFFF).expect("trailer");
    }
}
