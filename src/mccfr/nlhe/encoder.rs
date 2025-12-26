use super::*;
use crate::cards::*;
use crate::gameplay::*;
use crate::mccfr::*;
use std::collections::BTreeMap;

type NlheTree = Tree<Turn, Edge, Game, Info>;

#[derive(Default)]
pub struct NlheEncoder {
    lookup: BTreeMap<Isomorphism, Abstraction>,
}

impl NlheEncoder {
    pub fn abstraction(&self, iso: &Isomorphism) -> Abstraction {
        self.lookup
            .get(iso)
            .copied()
            .expect("isomorphsim not found in abstraction loookup")
    }
    pub fn choices(game: &Game, depth: usize) -> Vec<Edge> {
        Info::futures(game, depth).into_iter().collect()
    }
    pub fn raises(game: &Game, depth: usize) -> Vec<Odds> {
        Info::raises(game.street(), depth).to_vec()
    }
    pub fn unfold(game: &Game, depth: usize, action: Action) -> Vec<Edge> {
        Info::unfold(game, depth, action)
    }
}

impl crate::mccfr::Encoder for NlheEncoder {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Info;
    fn seed(&self, root: &Self::G) -> Self::I {
        Info::from_game(root, self)
    }
    fn info(&self, tree: &NlheTree, leaf: Branch<Self::E, Self::G>) -> Self::I {
        Info::from_tree(tree, leaf, self)
    }
}

#[cfg(feature = "database")]
impl crate::save::Schema for NlheEncoder {
    fn name() -> &'static str {
        crate::save::ISOMORPHISM
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT2,
        ]
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            crate::save::ISOMORPHISM,
            " (
                obs        BIGINT,
                abs        SMALLINT,
                position   INTEGER
            );"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ISOMORPHISM,
            "_covering     ON ",
            crate::save::ISOMORPHISM,
            " (obs, abs) INCLUDE (abs);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ISOMORPHISM,
            "_abs_position ON ",
            crate::save::ISOMORPHISM,
            " (abs, position);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ISOMORPHISM,
            "_abs_obs      ON ",
            crate::save::ISOMORPHISM,
            " (abs, obs);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ISOMORPHISM,
            "_abs          ON ",
            crate::save::ISOMORPHISM,
            " (abs);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ISOMORPHISM,
            "_obs          ON ",
            crate::save::ISOMORPHISM,
            " (obs);"
        )
    }
    fn copy() -> &'static str {
        const_format::concatcp!(
            "COPY ",
            crate::save::ISOMORPHISM,
            " (obs, abs) FROM STDIN BINARY"
        )
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", crate::save::ISOMORPHISM, ";")
    }
    // special freeze impl to sort. assign ordered numbers
    //  to each observation within each abstraction
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            crate::save::ISOMORPHISM,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            crate::save::ISOMORPHISM,
            " SET (autovacuum_enabled = false);
            WITH numbered AS (
                SELECT obs, abs, row_number()
                OVER (PARTITION BY abs ORDER BY obs) - 1 as rn
                FROM ",
            crate::save::ISOMORPHISM,
            "
            )
            UPDATE ",
            crate::save::ISOMORPHISM,
            "
            SET    position = numbered.rn
            FROM   numbered
            WHERE  ",
            crate::save::ISOMORPHISM,
            ".obs = numbered.obs
            AND    ",
            crate::save::ISOMORPHISM,
            ".abs = numbered.abs;"
        )
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl crate::save::Hydrate for NlheEncoder {
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        log::info!("loading isomorphism lookup from database");
        const SQL: &str =
            const_format::concatcp!("SELECT obs, abs FROM ", crate::save::ISOMORPHISM);
        let lookup = client
            .query(SQL, &[])
            .await
            .expect("isomorphism query")
            .into_iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, i16>(1)))
            .map(|(obs, abs)| (Isomorphism::from(obs), Abstraction::from(abs)))
            .collect::<BTreeMap<Isomorphism, Abstraction>>();
        Self { lookup }
    }
}

#[cfg(feature = "disk")]
#[allow(deprecated)]
impl crate::save::Disk for NlheEncoder {
    fn name() -> &'static str {
        crate::save::ISOMORPHISM
    }
    fn sources() -> Vec<std::path::PathBuf> {
        Street::all()
            .iter()
            .rev()
            .copied()
            .map(crate::clustering::Lookup::path)
            .collect()
    }
    fn save(&self) {
        unimplemented!("saving happens at Lookup level. composed of 4 street-level Lookup saves")
    }
    fn grow(_: Street) -> Self {
        unimplemented!("you have no business making an encoding from scratch, learn from kmeans")
    }
    fn load(_: Street) -> Self {
        Self {
            lookup: Street::all()
                .iter()
                .copied()
                .map(crate::clustering::Lookup::load)
                .map(BTreeMap::from)
                .fold(BTreeMap::default(), |mut map, l| {
                    map.extend(l);
                    map
                }),
        }
    }
}
