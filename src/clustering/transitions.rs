use crate::cards::street::Street;
use crate::clustering::histogram::Histogram;
use crate::gameplay::abstraction::Abstraction;
use std::collections::BTreeMap;
use std::mem::size_of;
use std::u16;

pub struct Shadow(BTreeMap<Abstraction, Histogram>);

impl From<BTreeMap<Abstraction, Histogram>> for Shadow {
    fn from(map: BTreeMap<Abstraction, Histogram>) -> Self {
        Self(map)
    }
}

impl Shadow {
    fn name() -> String {
        "transitions".to_string()
    }
}

impl crate::save::disk::Disk for Shadow {
    fn name() -> String {
        Self::name()
    }
    fn grow(street: Street) -> Self {
        unreachable!("you have no business making transition table from scratch {street}")
    }
    fn load(street: Street) -> Self {
        let ref path = Self::path(street);
        log::info!("{:<32}{:<32}", "loading     transitions", path.display());
        use byteorder::ReadBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let ref mass = street.n_children() as f32;
        let ref file = File::open(path).expect(&format!("open {}", path.display()));
        let mut decomp = BTreeMap::new();
        let mut reader = BufReader::new(file);
        let ref mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
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
                0xFFFF => break,
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
            .unwrap_or_else(|| Abstraction::from(0f32))
            .street();
        let ref path = Self::path(street);
        let ref mut file = File::create(path).expect(&format!("touch {}", path.display()));
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        log::info!("{:<32}{:<32}", "saving      transition", path.display());
        file.write_all(Self::header()).expect("header");
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
        file.write_u16::<BE>(Self::footer()).expect("trailer");
    }
}

impl crate::save::upload::Table for Shadow {
    fn name() -> String {
        Self::name()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8,
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
    fn creates() -> String {
        "
        CREATE TABLE IF NOT EXISTS transitions (
            prev       BIGINT,
            next       BIGINT,
            dx         REAL
        );"
        .to_string()
    }
    fn indices() -> String {
        "
        CREATE INDEX IF NOT EXISTS idx_transitions_dx        ON transitions(dx);
        CREATE INDEX IF NOT EXISTS idx_transitions_prev_dx   ON transitions(prev, dx);
        CREATE INDEX IF NOT EXISTS idx_transitions_next_dx   ON transitions(next, dx);
        CREATE INDEX IF NOT EXISTS idx_transitions_prev_next ON transitions(prev, next);
        CREATE INDEX IF NOT EXISTS idx_transitions_next_prev ON transitions(next, prev);
        "
        .to_string()
    }
    fn copy() -> String {
        "
        COPY transitions (
            prev,
            next,
            dx
        ) FROM STDIN BINARY
        "
        .to_string()
    }
}
