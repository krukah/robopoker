use super::{action::RpsEdge, info::RpsInfo, node::RpsNode, player::RpsPlayer};
use crate::cfr::training::tree::tree::Tree;
use std::{cell::RefCell, collections::HashSet};

pub(crate) struct RpsTree<'tree> {
    edges: RefCell<Vec<RpsEdge>>,
    nodes: RefCell<Vec<RpsNode<'tree>>>,
    infos: HashSet<RpsInfo<'tree>>,
}

impl<'t> RpsTree<'t> {
    pub fn new() -> Self {
        todo!("initialize game tree")
    }
}

impl<'t> Tree for RpsTree<'t> {
    fn infos(&self) -> Vec<&Self::TInfo> {
        self.infos.iter().collect()
    }

    type TPlayer = RpsPlayer;
    type TEdge = RpsEdge;
    type TNode = RpsNode<'t>;
    type TInfo = RpsInfo<'t>;
}
