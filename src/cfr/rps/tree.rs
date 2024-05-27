use super::{action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer};
use crate::cfr::training::tree::Tree;
use std::collections::HashSet;

/// Game tree
pub(crate) struct RPSTree<'tree> {
    edges: Vec<RPSEdge>,
    nodes: Vec<RPSNode<'tree>>,
    infos: HashSet<RPSInfo<'tree>>,
}

impl<'t> RPSTree<'t> {
    pub fn new() -> Self {
        todo!("initialize game tree")
    }
}
impl<'t> Tree for RPSTree<'t> {
    type TPlayer = RPSPlayer;
    type TEdge = RPSEdge;
    type TNode = RPSNode<'t>;
    type TInfo = RPSInfo<'t>;
    fn infos(&self) -> Vec<&Self::TInfo> {
        self.infos.iter().collect()
    }
}
