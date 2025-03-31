use super::branch::Branch;
use super::edge::Edge;
use super::game::Game;
use super::info::Info;
use super::node::Node;
use super::tree::Tree;
use super::turn::Turn;

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
pub trait Encoder<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn seed(&self, game: &G) -> I;
    fn info(&self, tree: &Tree<T, E, G, I>, leaf: Branch<E, G>) -> I;

    fn grow(&self, node: &Node<T, E, G, I>) -> Vec<Branch<E, G>> {
        node.branches()
    }
}
