use super::bucket::Bucket;
use super::data::Data;
use super::node::Node;
use super::spot::Spot;
use super::tree::Branch;
use super::tree::Tree;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::encoding::Encoder;
use crate::play::game::Game;

#[derive(Default)]
pub struct Sampler(Encoder);

impl Sampler {
    pub fn load() -> Self {
        Self(Encoder::load())
    }
    pub fn root(&self) -> Data {
        let game = Game::root();
        let info = self.abstraction(&game);
        Data::from((game, info))
    }
    pub fn abstraction(&self, game: &Game) -> Abstraction {
        self.0
            .abstraction(&Isomorphism::from(Observation::from(game)))
    }
    pub fn replay(&self, spot: &Spot) -> Tree {
        todo!()
    }
    pub fn bucket(&self, spot: &Spot) -> Bucket {
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
        node.outgoing()
            .into_iter()
            .cloned()
            .map(|e| (e, node.actionization(&e)))
            .map(|(e, a)| (e, node.data().game().apply(a))) // up to here should prolly be encapsulated by Node::children()
            .map(|(e, g)| (e, g, self.abstraction(&g)))
            .map(|(e, g, i)| (e, Data::from((g, i))))
            .map(|(e, d)| (e, d, node.index()))
            .map(|(e, d, n)| Branch(d, e, n))
            .collect()
    }
}
