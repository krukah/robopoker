use super::bucket::RpsBucket;
use super::tree::RpsTree;
use crate::cfr::rps::action::{Move, RpsAction};
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::Utility;

/// Shared-lifetime game tree nodes
pub(crate) struct RpsNode<'t> {
    tree: &'t RpsTree<'t>,

    index: usize,
    player: &'t RpsPlayer,

    parent: Option<usize>,
    childs: Vec<usize>,
    parent_edge: Option<RpsAction>,
    child_edges: Vec<RpsAction>,
}

impl<'t> RpsNode<'t> {
    pub fn new(tree: &'t RpsTree<'t>, index: usize, player: &'t RpsPlayer) -> Self {
        Self {
            tree,
            index,
            player,
            parent: None,
            parent_edge: None,
            childs: Vec::new(),
            child_edges: Vec::new(),
        }
    }
    pub fn bind(&'t mut self, child: &'t mut RpsNode<'t>) {
        self.childs.push(child.index());
        child.parent = Some(self.index());
    }
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Node for RpsNode<'_> {
    type NPlayer = RpsPlayer;
    type NAction = RpsAction;
    type NBucket = RpsBucket;

    fn utility(&self, player: &Self::NPlayer) -> Utility {
        const R_WIN: Utility = 1.0;
        const P_WIN: Utility = 1.0;
        const S_WIN: Utility = 1.0; // we can modify payoffs to verify convergence
        let a1 = self.parent_edge.expect("terminal node, depth > 1").turn();
        let a2 = self
            .parent()
            .expect("terminal node, depth = 2")
            .parent_edge()
            .expect("terminal node, depth = 2")
            .turn();
        let payoff = match (a1, a2) {
            (Move::R, Move::S) => R_WIN,
            (Move::R, Move::P) => -P_WIN,
            (Move::R, _) => 0.0,
            (Move::P, Move::R) => P_WIN,
            (Move::P, Move::S) => -S_WIN,
            (Move::P, _) => 0.0,
            (Move::S, Move::P) => S_WIN,
            (Move::S, Move::R) => -R_WIN,
            (Move::S, _) => 0.0,
        };
        let direction = match player {
            RpsPlayer::P1 => 0.0 + 1.0,
            RpsPlayer::P2 => 0.0 - 1.0,
        };
        direction * payoff
    }
    fn bucket(&self) -> Self::NBucket {
        RpsBucket {}
    }
    fn player(&self) -> &Self::NPlayer {
        self.player
    }
    fn parent(&self) -> Option<&Self> {
        self.parent.map(|i| self.tree.peek(i))
    }
    fn parent_edge(&self) -> Option<&Self::NAction> {
        self.parent_edge.as_ref()
    }
    fn children(&self) -> Vec<&Self> {
        self.childs.iter().map(|i| self.tree.peek(*i)).collect()
    }
    fn child_edges(&self) -> Vec<&Self::NAction> {
        self.childs
            .iter()
            .map(|i| self.tree.peek(*i).parent_edge().unwrap())
            .collect()
    }
}
