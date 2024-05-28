use super::{action::RpsAction, info::RpsInfo, node::RpsNode, player::RpsPlayer};
use crate::cfr::traits::tree::tree::Tree;
use std::{cell::RefCell, collections::HashSet};

pub(crate) struct RpsTree<'tree> {
    edges: RefCell<Vec<RpsAction>>,
    nodes: RefCell<Vec<RpsNode<'tree>>>,
    infos: HashSet<RpsInfo<'tree>>,
}

impl<'t> RpsTree<'t> {
    pub fn new() -> Self {
        // we first want to get root node
        // need method to use while let Some(node) = self.next()
        // probably useful to have Child(Node, Action) data structure
        // then we want to have method that takes     ref to node and returns Children
        // then we want to have method that takes mut ref to node and attaches Children / appends to self.nodes
        // recurse until all nodes are attached
        // during node iteration, map each to infoset vector
        todo!("initialize game tree")
    }
}

impl<'t> Tree for RpsTree<'t> {
    fn infos(&self) -> Vec<&Self::TInfo> {
        self.infos.iter().collect()
    }

    type TPlayer = RpsPlayer;
    type TEdge = RpsAction;
    type TNode = RpsNode<'t>;
    type TInfo = RpsInfo<'t>;
}
