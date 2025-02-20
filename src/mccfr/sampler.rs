use super::bucket::Bucket;
use super::data::Data;
use super::node::Node;
use super::recall::Recall;
use super::tree::Branch;
use super::tree::Tree;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::lookup::Lookup;
use crate::gameplay::game::Game;
use crate::Arbitrary;
use crate::Save;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Encoding(BTreeMap<Isomorphism, Abstraction>);

impl Encoding {
    pub fn seed(&self) -> Data {
        let game = Game::root();
        let info = self.abstraction(&game);
        Data::from((game, info))
    }
    pub fn abstraction(&self, game: &Game) -> Abstraction {
        self.0
            .get(&Isomorphism::from(game.sweat()))
            .cloned()
            .expect(&format!("precomputed abstraction missing for {game}"))
    }
    pub fn replay(&self, recall: &Recall) -> Tree {
        todo!("create a Tree from the vector of Actions in the Spot")
    }
    pub fn bucket(&self, recall: &Recall) -> Bucket {
        todo!("use some Recall-level function to get the list of Edge's by mapping (pseudoharmonically?) Actions -> Edges , w.r.t. Game::pot(). and then map over potential future actions w.r.t. Game::legal() to get another list of Edges. then use self.abstraction() to get the cluster lookup.
            impl From<Recall> for Path (history)
            impl From<Recall> for Path (choices)
            ...
            impl From<Recall> for (Path, Path)
            ");
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
        node.branches()
            .into_iter()
            .map(|(e, g)| (e, g, self.abstraction(&g)))
            .map(|(e, g, x)| (e, Data::from((g, x))))
            .map(|(e, d)| (e, d, node.index()))
            .map(|(e, d, n)| Branch(d, e, n))
            .collect()
    }
}

impl Save for Encoding {
    fn name() -> &'static str {
        unreachable!("saving happens at a lower level, composed of 4 street-level Lookup saves")
    }
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
                .iter()
                .copied()
                .map(Lookup::load)
                .map(BTreeMap::from)
                .fold(BTreeMap::default(), |mut map, l| {
                    map.extend(l);
                    map
                })
                .into(),
        )
    }
}

impl Arbitrary for Encoding {
    fn random() -> Self {
        const S: usize = 128;
        Self(
            (0..)
                .map(|_| Isomorphism::random())
                .map(|i| (i, Abstraction::random()))
                .filter(|(i, a)| i.0.street() == a.street())
                .take(S)
                .collect::<BTreeMap<_, _>>()
                .into(),
        )
    }
}
