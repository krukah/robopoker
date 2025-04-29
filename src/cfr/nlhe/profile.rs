use crate::cards::street::Street;
use crate::cfr::nlhe::edge::Edge;
use crate::cfr::nlhe::game::Game;
use crate::cfr::nlhe::info::Info;
use crate::cfr::nlhe::turn::Turn;
use std::collections::BTreeMap;

type Node<'a> = crate::cfr::structs::node::Node<'a, Turn, Edge, Game, Info>;
type Branch = crate::cfr::types::branch::Branch<Edge, Game>;

#[derive(Default)]
pub struct Profile {
    iterations: usize,
    encounters: BTreeMap<Info, BTreeMap<Edge, (crate::Probability, crate::Utility)>>,
}

impl Profile {
    pub fn at(&mut self, info: Info, edge: Edge) -> &mut (crate::Probability, crate::Utility) {
        self.encounters
            .entry(info)
            .or_insert_with(BTreeMap::default)
            .entry(edge)
            .or_insert_with(|| (0., 0.))
    }
    /// uniform sampling of chance Edge
    fn explore_any(&self, choices: Vec<Branch>, head: &Node) -> Vec<Branch> {
        use crate::cfr::traits::profile::Profile;
        use rand::Rng;
        let n = choices.len();
        let mut choices = choices;
        let ref mut rng = self.rng(head.info());
        let choice = rng.gen_range(0..n);
        let chosen = choices.remove(choice);
        vec![chosen]
    }
    /// Profile-weighted sampling of opponent Edge
    fn explore_one(&self, choices: Vec<Branch>, head: &Node) -> Vec<Branch> {
        use crate::cfr::traits::profile::Profile;
        use rand::distributions::WeightedIndex;
        use rand::prelude::Distribution;
        let ref info = head.info();
        let ref mut rng = self.rng(info);
        let mut choices = choices;
        let policy = choices
            .iter()
            .map(|(edge, _, _)| self.policy(info, edge))
            .collect::<Vec<_>>();
        let choice = WeightedIndex::new(policy)
            .expect("at least one policy > 0")
            .sample(rng);
        let chosen = choices.remove(choice);
        vec![chosen]
    }

    fn name() -> String {
        "blueprint".to_string()
    }
}

impl crate::cfr::traits::profile::Profile for Profile {
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
    fn weight(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(w, _)| *w)
            .unwrap_or_default()
    }
    fn regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }
    fn sample(&self, node: &Node, branches: Vec<Branch>) -> Vec<Branch> {
        let n = branches.len();
        let p = node.game().turn();
        let walker = self.walker();
        let chance = Turn::Chance;
        match (n, p) {
            (0, _) => branches,
            (_, p) if p == walker => branches,
            (_, p) if p == chance => self.explore_any(branches, node),
            (_, p) if p != walker => self.explore_one(branches, node),
            _ => panic!("at the disco"),
        }
    }
}

#[cfg(feature = "native")]
impl crate::save::upload::Table for Profile {
    fn name() -> String {
        Self::name()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::FLOAT4,
            tokio_postgres::types::Type::FLOAT4,
        ]
    }
    fn sources() -> Vec<String> {
        use crate::save::disk::Disk;
        use crate::Arbitrary;
        vec![Self::path(Street::random())]
    }
    fn copy() -> String {
        "COPY blueprint (
            past,
            present,
            future,
            edge,
            policy,
            regret
        )
        FROM STDIN BINARY
        "
        .to_string()
    }
    fn creates() -> String {
        "
        CREATE TABLE IF NOT EXISTS blueprint (
            edge       BIGINT,
            past       BIGINT,
            present    BIGINT,
            future     BIGINT,
            policy     REAL,
            regret     REAL
        );
        "
        .to_string()
    }
    fn indices() -> String {
        "
        CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
        CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
        CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
        CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
        CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
        "
        .to_string()
    }
}

impl crate::save::disk::Disk for Profile {
    fn name() -> String {
        Self::name()
    }
    fn grow(_: Street) -> Self {
        unreachable!("must be learned in MCCFR minimization")
    }
    fn path(_: Street) -> String {
        format!(
            "{}/pgcopy/{}",
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            Self::name()
        )
    }
    fn load(_: Street) -> Self {
        let ref path = Self::path(Street::random());
        log::info!("{:<32}{:<32}", "loading     blueprint", path);
        use crate::cfr::nlhe::info::Info;
        use crate::clustering::abstraction::Abstraction;
        use crate::gameplay::path::Path;
        use crate::Arbitrary;
        use byteorder::ReadBytesExt;
        use byteorder::BE;
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
                    let present = Abstraction::from(reader.read_u64::<BE>().expect("abstraction"));
                    reader.read_u32::<BE>().expect("future path length");
                    let choices = Path::from(reader.read_u64::<BE>().expect("choices"));
                    reader.read_u32::<BE>().expect("edge length");
                    let edge = Edge::from(reader.read_u64::<BE>().expect("read edge"));
                    reader.read_u32::<BE>().expect("regret length");
                    let regret = reader.read_f32::<BE>().expect("read regret");
                    reader.read_u32::<BE>().expect("policy length");
                    let policy = reader.read_f32::<BE>().expect("read policy");
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
        }
    }
    fn save(&self) {
        const N_FIELDS: u16 = 6;
        let ref path = Self::path(Street::random());
        let ref mut file = File::create(path).expect(&format!("touch {}", path));
        use crate::Arbitrary;
        use byteorder::WriteBytesExt;
        use byteorder::BE;
        use std::fs::File;
        use std::io::Write;
        log::info!("{:<32}{:<32}", "saving      blueprint", path);
        file.write_all(Self::header()).expect("header");
        for (bucket, strategy) in self.encounters.iter() {
            for (edge, memory) in strategy.iter() {
                file.write_u16::<BE>(N_FIELDS).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(*bucket.history())).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(*bucket.present())).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(*bucket.futures())).unwrap();
                file.write_u32::<BE>(size_of::<u64>() as u32).unwrap();
                file.write_u64::<BE>(u64::from(edge.clone())).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(memory.1).unwrap();
                file.write_u32::<BE>(size_of::<f32>() as u32).unwrap();
                file.write_f32::<BE>(memory.0).unwrap();
            }
        }
        file.write_u16::<BE>(Self::footer()).expect("trailer");
    }
}
