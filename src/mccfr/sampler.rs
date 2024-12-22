use super::bucket::Bucket;
use super::data::Data;
use super::node::Node;
use super::spot::Spot;
use super::tree::Branch;
use super::tree::Tree;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::layer::Layer;
use crate::clustering::lookup::Lookup;
use crate::gameplay::game::Game;
use crate::Arbitrary;
use crate::Save;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Sampler(BTreeMap<Isomorphism, Abstraction>);

impl Sampler {
    /// all-in-one entry point for learning the kmeans abstraction and
    /// writing to disk in pgcopy
    pub fn learn() {
        Street::all()
            .iter()
            .filter(|s| !Layer::done(**s))
            .map(|s| Layer::make(*s))
            .count();
    }
    pub fn root(&self) -> Data {
        let game = Game::root();
        let info = self.abstraction(&game);
        Data::from((game, info))
    }
    pub fn abstraction(&self, game: &Game) -> Abstraction {
        self.0
            .get(&Isomorphism::from(Observation::from(game)))
            .cloned()
            .expect(&format!("precomputed abstraction missing for {game}"))
    }
    pub fn replay(&self, _: &Spot) -> Tree {
        todo!()
    }
    pub fn bucket(&self, _: &Spot) -> Bucket {
        todo!();
    }

    /// unfiltered set of possible children of a Node,
    /// conditional on its History (# raises, street granularity).
    /// the head Node is attached to the Tree stack-recursively,
    /// while the leaf Data is generated here with help from Sampler.
    /// Rust's ownership makes this a bit awkward but for very good reason!
    /// It has forced me to decouple global (Path) from local (Data)
    /// properties of Tree sampling, which makes lots of sense and is stronger model.
    /// broadly goes from Edge -> Action -> Game -> Abstraction
    pub fn branches(&self, node: &Node) -> Vec<Branch> {
        node.continuations()
            .iter()
            .map(|e| (e, node.actionization(e)))
            .map(|(e, a)| (e.clone(), node.data().game().apply(a))) // up to here should prolly be encapsulated by Node::children()
            .map(|(e, g)| (e, g, self.abstraction(&g)))
            .map(|(e, g, x)| (e, Data::from((g, x))))
            .map(|(e, d)| (e, d, node.index()))
            .map(|(e, d, n)| Branch(d, e, n))
            .collect()
    }
}

impl Save for Sampler {
    fn save(&self) {
        unreachable!("saving happens at a lower level, composed of 4 street-level Lookup saves")
    }
    fn make(_: Street) -> Self {
        unreachable!("you have no buisiness making an encoding from scratch")
    }
    fn done(_: Street) -> bool {
        Street::all()
            .iter()
            .copied()
            .all(|street| Lookup::done(street))
    }
    fn load(_: Street) -> Self {
        Self(
            Street::all()
                .into_iter()
                .copied()
                .map(|s| Lookup::load(s))
                .map(|l| BTreeMap::from(l))
                .fold(BTreeMap::default(), |mut map, l| {
                    map.extend(l);
                    map
                })
                .into(),
        )
    }
}

impl Arbitrary for Sampler {
    fn random() -> Self {
        Self(
            (0..100)
                .map(|_| Isomorphism::random())
                .map(|i| (i, Abstraction::random()))
                .collect::<BTreeMap<_, _>>()
                .into(),
        )
    }
}
