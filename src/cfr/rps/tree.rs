use super::{action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer};
use crate::cfr::training::tree::tree::Tree;
use std::{cell::RefCell, collections::HashSet};

pub(crate) struct RPSTree<'tree> {
    edges: RefCell<Vec<RPSEdge>>,
    nodes: RefCell<Vec<RPSNode<'tree>>>,
    infos: HashSet<RPSInfo<'tree>>,
}

impl<'t> RPSTree<'t> {
    pub fn new() -> Self {
        todo!("initialize game tree")
    }
}

impl<'t> Tree for RPSTree<'t> {
    fn infos(&self) -> Vec<&Self::TInfo> {
        self.infos.iter().collect()
    }

    type TPlayer = RPSPlayer;
    type TEdge = RPSEdge;
    type TNode = RPSNode<'t>;
    type TInfo = RPSInfo<'t>;
}
