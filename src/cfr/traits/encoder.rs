use super::edge::Edge;
use super::game::Game;
use super::info::Info;
use super::turn::Turn;
use crate::cfr::structs::node::Node;
use crate::cfr::structs::tree::Tree;
use crate::cfr::types::branch::Branch;

/// infoset encoding is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the encoder must be able to create an Info from:
///  a Game
///  a Game, an Edge, and the parent Node for topological context
///
/// some implemenstaions may not need to reference the parent Node,
/// RPS for example has trivial infoset encoding,
/// whereas NLHE must learn the massive abstraction from kmeans clustering
/// over the set of all hands up to strategic isomorphism.
pub trait Encoder {
    type T: Turn;
    type E: Edge;
    type G: Game<E = Self::E, T = Self::T>;
    type I: Info<E = Self::E, T = Self::T>;

    fn seed(&self, game: &Self::G) -> Self::I;
    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        leaf: Branch<Self::E, Self::G>,
    ) -> Self::I;

    /// because we assume both that Game's can
    /// be computed from applying Edge's, and that
    /// the Node must have access to its Info set,
    /// we can delegate the branching logic to the Node itself,
    /// which already has all the necessary information to compute
    /// the valid edges and resulting game states.
    fn branches(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        node.branches()
    }
}
