use crate::cards::isomorphism::Isomorphism;
use crate::cards::street::Street;
use crate::cfr::nlhe::edge::Edge;
use crate::cfr::nlhe::game::Game;
use crate::cfr::nlhe::info::Info;
use crate::cfr::nlhe::turn::Turn;
use crate::cfr::types::branch::Branch;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::Lookup;
use crate::gameplay::action::Action;
use crate::gameplay::path::Path;
use std::collections::BTreeMap;

type Tree = crate::cfr::structs::tree::Tree<Turn, Edge, Game, Info>;

#[derive(Default)]
pub struct Sampler {
    lookup: BTreeMap<Isomorphism, Abstraction>,
}

impl Sampler {
    fn name() -> String {
        "isomorphism".to_string()
    }

    pub fn abstraction(&self, iso: &Isomorphism) -> Abstraction {
        self.lookup
            .get(iso)
            .copied()
            .expect("isomorphsim not found in abstraction loookup")
    }

    pub fn choices(game: &Game, depth: usize) -> Vec<crate::cfr::nlhe::edge::Edge> {
        game.legal()
            .into_iter()
            .flat_map(|action| Self::unfold(game, depth, action))
            .collect()
    }

    pub fn raises(game: &Game, depth: usize) -> Vec<crate::gameplay::odds::Odds> {
        if depth > crate::MAX_RAISE_REPEATS {
            vec![]
        } else {
            match game.street() {
                Street::Pref => crate::gameplay::odds::Odds::PREF_RAISES.to_vec(),
                Street::Flop => crate::gameplay::odds::Odds::FLOP_RAISES.to_vec(),
                _ => match depth {
                    0 => crate::gameplay::odds::Odds::LATE_RAISES.to_vec(),
                    _ => crate::gameplay::odds::Odds::LAST_RAISES.to_vec(),
                },
            }
        }
    }

    pub fn unfold(game: &Game, depth: usize, action: Action) -> Vec<crate::cfr::nlhe::edge::Edge> {
        match action {
            Action::Raise(_) => Self::raises(game, depth)
                .into_iter()
                .map(crate::cfr::nlhe::edge::Edge::from)
                .collect::<Vec<crate::cfr::nlhe::edge::Edge>>(),
            _ => vec![crate::cfr::nlhe::edge::Edge::from(action)],
        }
    }

    #[allow(dead_code)]
    fn infoize(&self, recall: &crate::gameplay::recall::Recall) -> Info {
        let depth = 0;
        let ref game = recall.head();
        let ref iso = recall.isomorphism();
        let present = self.abstraction(iso);
        let futures = Path::from(Self::choices(game, depth));
        let history = Path::from(recall.path());
        Info::from((history, present, futures))
    }
}

impl crate::cfr::traits::sampler::Sampler for Sampler {
    type T = crate::cfr::nlhe::turn::Turn;
    type E = crate::cfr::nlhe::edge::Edge;
    type G = crate::cfr::nlhe::game::Game;
    type I = crate::cfr::nlhe::info::Info;

    fn seed(&self, root: &Self::G) -> Self::I {
        let ref iso = Isomorphism::from(root.sweat());
        let depth = 0;
        let present = self.abstraction(iso);
        let history = Path::default();
        let futures = Path::from(Self::choices(root, depth));
        Self::I::from((history, present, futures))
    }
    fn info(&self, tree: &Tree, leaf: Branch<Self::E, Self::G>) -> Self::I {
        let (edge, ref game, head) = leaf;
        let head = tree.at(head);
        let ref iso = Isomorphism::from(game.sweat());
        let n_raises = head
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count();
        let present = self.abstraction(iso);
        let futures = Path::from(Self::choices(game, n_raises));
        let history = std::iter::once(edge).chain(head).collect::<Path>();
        Self::I::from((history, present, futures))
    }
}

#[cfg(feature = "native")]
impl crate::save::upload::Table for Sampler {
    fn name() -> String {
        Self::name()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8,
            tokio_postgres::types::Type::INT8,
        ]
    }
    fn sources() -> Vec<String> {
        use crate::save::disk::Disk;
        Street::all()
            .iter()
            .rev()
            .copied()
            .map(Lookup::path)
            .collect()
    }
    fn creates() -> String {
        "
            CREATE TABLE IF NOT EXISTS isomorphism (
                obs        BIGINT,
                abs        BIGINT,
                position   INTEGER
            );"
        .to_string()
    }
    fn indices() -> String {
        "
            CREATE INDEX IF NOT EXISTS idx_isomorphism_covering     ON isomorphism  (obs, abs) INCLUDE (abs);
            CREATE INDEX IF NOT EXISTS idx_isomorphism_abs_position ON isomorphism  (abs, position);
            CREATE INDEX IF NOT EXISTS idx_isomorphism_abs_obs      ON isomorphism  (abs, obs);
            CREATE INDEX IF NOT EXISTS idx_isomorphism_abs          ON isomorphism  (abs);
            CREATE INDEX IF NOT EXISTS idx_isomorphism_obs          ON isomorphism  (obs);
            --
            WITH numbered AS (
                SELECT obs, abs, row_number() OVER (PARTITION BY abs ORDER BY obs) - 1 as rn
                FROM isomorphism
            )
                UPDATE isomorphism
                SET    position = numbered.rn
                FROM   numbered
                WHERE  isomorphism.obs = numbered.obs
                AND    isomorphism.abs = numbered.abs;
            "
            .to_string()
    }
    fn copy() -> String {
        "
            COPY isomorphism (
                obs,
                abs
            )
            FROM STDIN BINARY
            "
        .to_string()
    }
}

impl crate::save::disk::Disk for Sampler {
    fn name() -> String {
        Self::name()
    }
    fn save(&self) {
        unimplemented!("saving happens at Lookup level. composed of 4 street-level Lookup saves")
    }
    fn grow(_: Street) -> Self {
        unimplemented!("you have no business making an encoding from scratch, learn from kmeans")
    }
    fn load(_: Street) -> Self {
        let lookup = Street::all()
            .iter()
            .copied()
            .map(crate::clustering::lookup::Lookup::load)
            .map(BTreeMap::from)
            .fold(BTreeMap::default(), |mut map, l| {
                map.extend(l);
                map
            })
            .into();
        Self { lookup }
    }
}
