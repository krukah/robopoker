use crate::cards::isomorphism::Isomorphism;
use crate::cards::street::Street;
use crate::cfr::nlhe::edge::Edge;
use crate::cfr::nlhe::game::Game;
use crate::cfr::nlhe::info::Info;
use crate::cfr::nlhe::turn::Turn;
use crate::cfr::types::branch::Branch;
use crate::clustering::abstraction::Abstraction;
use crate::gameplay::action::Action;
use crate::gameplay::path::Path;
use std::collections::BTreeMap;

type Tree = crate::cfr::structs::tree::Tree<Turn, Edge, Game, Info>;

#[derive(Default)]
pub struct Sampler {
    lookup: BTreeMap<Isomorphism, Abstraction>,
}

impl Sampler {
    fn abstraction(&self, iso: &Isomorphism) -> Abstraction {
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

    fn raises(game: &Game, depth: usize) -> Vec<crate::gameplay::odds::Odds> {
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

    fn unfold(game: &Game, depth: usize, action: Action) -> Vec<crate::cfr::nlhe::edge::Edge> {
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
        let history = Path::from(recall.history());
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
        crate::clustering::lookup::Lookup::name()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        crate::clustering::lookup::Lookup::columns()
    }
    fn sources() -> Vec<String> {
        crate::clustering::lookup::Lookup::sources()
    }
    fn creates() -> String {
        crate::clustering::lookup::Lookup::creates()
    }
    fn indices() -> String {
        crate::clustering::lookup::Lookup::indices()
    }
    fn copy() -> String {
        crate::clustering::lookup::Lookup::copy()
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
    fn save(&self) {
        unimplemented!("saving happens at Lookup level. composed of 4 street-level Lookup saves")
    }
    fn grow(_: Street) -> Self {
        unimplemented!("you have no business making an encoding from scratch, learn from kmeans")
    }
}
