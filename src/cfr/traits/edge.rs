impl Turn for petgraph::graph::EdgeIndex {}
impl Turn for crate::gameplay::action::Action {}
impl Turn for crate::gameplay::edge::Edge {}

impl Decision<crate::gameplay::edge::Edge> for crate::mccfr::path::Path {
    fn choices(&self) -> impl Iterator<Item = crate::gameplay::edge::Edge> {
        self.into_iter()
    }
}

/// marker trait for things that can happen between games
pub trait Turn: Copy + Clone + PartialEq + Eq + std::fmt::Debug {}

trait Player: Clone + Copy + PartialEq + Eq {
    fn chance() -> Self;
}

trait Game: Clone + Copy {
    type E: Turn;
    type W: Player;
    fn root() -> Self;
    fn turn(&self) -> Self::W;
    fn payoff(&self, player: Self::W) -> crate::Utility;
}

pub trait Decision<E>: Clone + Copy + PartialEq + Eq
where
    E: Turn,
{
    fn choices(&self) -> impl Iterator<Item = E>;
}

struct Tree<'tree, G, T, P>
where
    G: Game,
    T: Turn,
    P: Player,
{
    index: petgraph::graph::NodeIndex,
    graph: &'tree petgraph::graph::DiGraph<G, T>,
    phantom: std::marker::PhantomData<P>,
}
