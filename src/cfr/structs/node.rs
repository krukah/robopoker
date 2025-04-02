use crate::cfr::traits::edge::Edge;
use crate::cfr::traits::game::Game;
use crate::cfr::traits::info::Info;
use crate::cfr::traits::turn::Turn;
use crate::cfr::types::branch::Branch;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;

/// the node is pre-implemented. it is a wrapper around
/// a NodeIndex, and a thread-safe readonly reference
/// to the Tree in which it resides.
///
/// by only assuming the tree property of the underlying graph,
/// we can implement navigation methods recursively. all while being
/// fully generic over Turn Edge Game Info. just that they need to be
#[derive(Copy, Clone)]
pub struct Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    index: NodeIndex,
    graph: &'tree DiGraph<(G, I), E>,
    danny: std::marker::PhantomData<(T, I)>,
}

impl<'tree, T, E, G, I> Iterator for Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        let (ref mut parent, incoming) = self.up()?;
        std::mem::swap(self, parent);
        Some(incoming.clone())
    }
}

impl<'tree, T, E, G, I> Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    pub fn from(index: NodeIndex, graph: &'tree DiGraph<(G, I), E>) -> Self {
        Self {
            index,
            graph,
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }
    pub fn index(&self) -> NodeIndex {
        self.index
    }
    pub fn graph(&self) -> &'tree DiGraph<(G, I), E> {
        self.graph
    }
    pub fn game(&self) -> &G {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid game index")
            .0
    }
    pub fn info(&self) -> &I {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid info index")
            .1
    }
    pub fn at(&self, index: NodeIndex) -> Node<'tree, T, E, G, I> {
        Self {
            index,
            graph: self.graph(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }

    pub fn up(&self) -> Option<(Node<'tree, T, E, G, I>, &'tree E)> {
        match (self.parent(), self.incoming()) {
            (Some(parent), Some(incoming)) => Some((parent, incoming)),
            (Some(_), _) => unreachable!("tree property violation"),
            (_, Some(_)) => unreachable!("tree property violation"),
            (None, None) => None,
        }
    }
    pub fn parent(&self) -> Option<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|index| self.at(index))
    }
    pub fn incoming(&self) -> Option<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|edge| edge.weight())
    }
    pub fn follow(&self, edge: &E) -> Option<Node<'tree, T, E, G, I>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.at(child.index()))
    }
    pub fn outgoing(&self) -> Vec<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|edge| edge.weight())
            .collect()
    }
    pub fn children(&self) -> Vec<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|index| self.at(index))
            .collect()
    }
    pub fn descendants(&self) -> Vec<Node<'tree, T, E, G, I>> {
        if self.children().is_empty() {
            vec![self.clone()]
        } else {
            self.children()
                .into_iter()
                .map(|child| child.descendants())
                .flatten()
                .collect()
        }
    }
    pub fn branches(&self) -> Vec<Branch<E, G>> {
        self.info()
            .choices()
            .into_iter()
            .map(|e| (e.clone(), self.game().apply(e), self.index()))
            .collect()
    }
}
