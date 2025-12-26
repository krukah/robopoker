use super::*;
use crate::gameplay::*;
use crate::mccfr::*;
use crate::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct NlheProfile {
    pub iterations: usize,
    pub encounters: BTreeMap<Info, BTreeMap<Edge, (Probability, Utility)>>,
    pub metrics: Metrics,
}

impl Profile for NlheProfile {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Info;

    fn increment(&mut self) {
        self.iterations += 1;
    }
    fn walker(&self) -> Self::T {
        match self.iterations % 2 {
            0 => Turn::Choice(0),
            _ => Turn::Choice(1),
        }
    }
    fn epochs(&self) -> usize {
        self.iterations
    }
    fn metrics(&self) -> Option<&Metrics> {
        Some(&self.metrics)
    }
    fn sum_policy(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(w, _)| *w)
            .unwrap_or_default()
    }
    fn sum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }
}

#[cfg(feature = "database")]
impl crate::save::Schema for NlheProfile {
    fn name() -> &'static str {
        crate::save::BLUEPRINT
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT2,
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::FLOAT4,
            tokio_postgres::types::Type::FLOAT4,
        ]
    }
    fn copy() -> &'static str {
        const_format::concatcp!(
            "COPY ",
            crate::save::BLUEPRINT,
            " (past, present, future, edge, policy, regret) FROM STDIN BINARY"
        )
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            crate::save::BLUEPRINT,
            " (
                edge       BIGINT,
                past       BIGINT,
                present    SMALLINT,
                future     BIGINT,
                policy     REAL,
                regret     REAL,
                UNIQUE     (past, present, future, edge)
            );"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_blueprint_upsert  ON ",
            crate::save::BLUEPRINT,
            " (present, past, future, edge);
             CREATE        INDEX IF NOT EXISTS idx_blueprint_bucket  ON ",
            crate::save::BLUEPRINT,
            " (present, past, future);
             CREATE        INDEX IF NOT EXISTS idx_blueprint_future  ON ",
            crate::save::BLUEPRINT,
            " (future);
             CREATE        INDEX IF NOT EXISTS idx_blueprint_present ON ",
            crate::save::BLUEPRINT,
            " (present);
             CREATE        INDEX IF NOT EXISTS idx_blueprint_edge    ON ",
            crate::save::BLUEPRINT,
            " (edge);
             CREATE        INDEX IF NOT EXISTS idx_blueprint_past    ON ",
            crate::save::BLUEPRINT,
            " (past);"
        )
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", crate::save::BLUEPRINT, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            crate::save::BLUEPRINT,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            crate::save::BLUEPRINT,
            " SET (autovacuum_enabled = false);"
        )
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl crate::save::Hydrate for NlheProfile {
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        log::info!("loading blueprint from database");
        // grab current epoch from metadata
        const EPOCH_SQL: &str = const_format::concatcp!(
            "SELECT value FROM ",
            crate::save::EPOCH,
            " WHERE key = 'current'"
        );
        let iterations = client
            .query_opt(EPOCH_SQL, &[])
            .await
            .ok()
            .flatten()
            .map(|r| r.get::<_, i64>(0) as usize)
            .expect("to have already created epoch metadata");
        // iterate over rows
        const BLUEPRINT_SQL: &str = const_format::concatcp!(
            "SELECT past, present, future, edge, policy, regret FROM ",
            crate::save::BLUEPRINT
        );
        let mut encounters = BTreeMap::new();
        for row in client
            .query(BLUEPRINT_SQL, &[])
            .await
            .expect("to have already created blueprint")
        {
            let history = Path::from(row.get::<_, i64>(0) as u64);
            let present = Abstraction::from(row.get::<_, i16>(1));
            let choices = Path::from(row.get::<_, i64>(2) as u64);
            let edge = Edge::from(row.get::<_, i64>(3) as u64);
            let policy = row.get::<_, f32>(4);
            let regret = row.get::<_, f32>(5);
            let bucket = Info::from((history, present, choices));
            encounters
                .entry(bucket)
                .or_insert_with(BTreeMap::default)
                .entry(edge)
                .or_insert((policy, regret));
        }
        log::info!("loaded {} infos from database", encounters.len());
        Self {
            iterations,
            encounters,
            metrics: Metrics::default(),
        }
    }
}

#[cfg(feature = "database")]
impl NlheProfile {
    pub fn rows(self) -> impl Iterator<Item = (i64, i16, i64, i64, f32, f32)> {
        self.encounters.into_iter().flat_map(|(info, edges)| {
            let history = i64::from(*info.history());
            let present = i16::from(*info.present());
            let choices = i64::from(*info.choices());
            edges
                .into_iter()
                .map(move |(e, (p, r))| (u64::from(e) as i64, p, r))
                .map(move |(e, p, r)| (history, present, choices, e, p, r))
        })
    }
}

#[cfg(feature = "disk")]
use crate::cards::*;

#[cfg(feature = "disk")]
#[allow(deprecated)]
impl crate::save::Disk for NlheProfile {
    fn name() -> &'static str {
        crate::save::BLUEPRINT
    }
    fn sources() -> Vec<std::path::PathBuf> {
        vec![Self::path(Street::random())]
    }
    fn grow(_: Street) -> Self {
        unreachable!("must be learned in MCCFR minimization")
    }
    fn path(_: Street) -> std::path::PathBuf {
        let ref path = format!(
            "{}/pgcopy/{}",
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            crate::save::BLUEPRINT
        );
        std::path::Path::new(path).parent().map(std::fs::create_dir);
        std::path::PathBuf::from(path)
    }
    fn load(_: Street) -> Self {
        let ref path = Self::path(Street::random());
        log::info!("{:<32}{:<32}", "loading     blueprint", path.display());
        use byteorder::BE;
        use byteorder::ReadBytesExt;
        use std::fs::File;
        use std::io::BufReader;
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        let file = File::open(path).expect("open file");
        let mut encounters = BTreeMap::new();
        let mut reader = BufReader::new(file);
        let ref mut buffer = [0u8; 2];
        reader.seek(SeekFrom::Start(19)).expect("seek past header");
        while reader.read_exact(buffer).is_ok() {
            match u16::from_be_bytes(buffer.clone()) {
                6 => {
                    reader.read_u32::<BE>().expect("past path length");
                    let history = Path::from(reader.read_u64::<BE>().expect("history"));
                    reader.read_u32::<BE>().expect("abstraction length");
                    let present = Abstraction::from(reader.read_i16::<BE>().expect("abstraction"));
                    reader.read_u32::<BE>().expect("future path length");
                    let choices = Path::from(reader.read_u64::<BE>().expect("choices"));
                    reader.read_u32::<BE>().expect("edge length");
                    let edge = Edge::from(reader.read_u64::<BE>().expect("read edge"));
                    reader.read_u32::<BE>().expect("policy length");
                    let policy = reader.read_f32::<BE>().expect("read policy");
                    reader.read_u32::<BE>().expect("regret length");
                    let regret = reader.read_f32::<BE>().expect("read regret");
                    let bucket = Info::from((history, present, choices));
                    encounters
                        .entry(bucket)
                        .or_insert_with(BTreeMap::default)
                        .entry(edge)
                        .or_insert((policy, regret));
                }
                0xFFFF => break,
                n => panic!("unexpected number of fields: {}", n),
            }
        }
        Self {
            encounters,
            iterations: 0,
            metrics: Metrics::default(),
        }
    }
    fn save(&self) {
        const N_FIELDS: u16 = 6;
        let ref path = Self::path(Street::random());
        let ref mut file = File::create(path).expect(&format!("touch {}", path.display()));
        use byteorder::BE;
        use byteorder::WriteBytesExt;
        use std::fs::File;
        use std::io::Write;
        log::info!("{:<32}{:<32}", "saving      blueprint", path.display());
        file.write_all(Self::header()).expect("header");
        for (bucket, strategy) in self.encounters.iter() {
            for (edge, memory) in strategy.iter() {
                file.write_u16::<BE>(N_FIELDS).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(*bucket.history())).unwrap();
                file.write_u32::<BE>(size_of::<i16>() as u32).unwrap();
                file.write_i16::<BE>(i16::from(*bucket.present())).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(*bucket.choices())).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(edge.clone())).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(memory.0).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(memory.1).unwrap();
            }
        }
        file.write_u16::<BE>(Self::footer()).expect("trailer");
    }
}
