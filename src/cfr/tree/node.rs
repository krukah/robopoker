use crate::cfr::traits::{action::E, bucket::B, local::L, player::C};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    Direction::{Incoming, Outgoing},
};
use std::ptr::NonNull;

pub(crate) struct N {
    pub graph: NonNull<DiGraph<Self, E>>,
    pub index: NodeIndex,
    pub local: L,
}

/// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
impl N {
    // private
    fn graph(&self) -> &DiGraph<Self, E> {
        unsafe { self.graph.as_ref() }
    }
    // observability
    pub fn local(&self) -> &L {
        &self.local
    }
    pub fn index(&self) -> &NodeIndex {
        &self.index
    }
    pub fn bucket(&self) -> &B {
        self.local().bucket()
    }
    pub fn player(&self) -> &C {
        self.local().player()
    }
    pub fn payoff(&self, player: &C) -> crate::Utility {
        self.local().payoff(player)
    }
    // walkability
    pub fn incoming(&self) -> Option<&E> {
        self.graph()
            .edges_directed(*self.index(), Incoming)
            .next()
            .map(|e| e.weight())
    }
    pub fn outgoing(&self) -> Vec<&E> {
        self.graph()
            .edges_directed(*self.index(), Outgoing)
            .map(|e| e.weight())
            .collect()
    }
    pub fn parent<'a>(&'a self) -> Option<&'a Self> {
        self.graph()
            .neighbors_directed(*self.index(), Incoming)
            .next()
            .map(|index| {
                self.graph()
                    .node_weight(index)
                    .expect("tree property: if incoming edge, then parent")
            })
    }
    pub fn children<'a>(&'a self) -> Vec<&'a Self> {
        self.graph()
            .neighbors_directed(*self.index(), Outgoing)
            .map(|c| {
                self.graph()
                    .node_weight(c)
                    .expect("tree property: if outgoing edge, then child")
            })
            .collect()
    }
    pub fn descendants<'a>(&'a self) -> Vec<&'a Self> {
        match self.children().len() {
            0 => vec![&self],
            _ => self
                .children()
                .iter()
                .map(|child| child.descendants())
                .flatten()
                .collect(),
        }
    }
    pub fn follow<'a>(&'a self, edge: &E) -> &'a Self {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .unwrap()
    }
}
