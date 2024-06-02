#![allow(dead_code)]
use super::bucket::RpsBucket;
use crate::cfr::rps::action::{Move, RpsAction};
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::Utility;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::{Incoming, Outgoing};
use std::ptr::NonNull;

/// arbitrary inner data structure. used to spawn and evaluate game-specific rules
pub(crate) struct RpsInner;
impl RpsInner {
    pub fn player(&self) -> &RpsPlayer {
        todo!()
    }
    pub fn spawn(&self) -> Vec<(RpsInner, RpsAction)> {
        todo!()
    }
}

/// a node equipped with full tree access w unsafe pointers, allowing for local traversal i.e. implementing our CfrNode trait
pub(crate) struct RpsNode {
    inner: RpsInner,
    index: NodeIndex,
    graph: NonNull<DiGraph<RpsNode, RpsAction>>, // allow for unsafe circular reference
                                                 // to graph to enable local traversal
}
impl RpsNode {
    /// SAFETY: idk tbh
    fn graph(&self) -> &DiGraph<Self, RpsAction> {
        unsafe { self.graph.as_ref() }
    }
    pub fn inner(&self) -> &RpsInner {
        &self.inner
    }
    pub fn new(
        inner: RpsInner,
        index: NodeIndex,
        graph: NonNull<DiGraph<Self, RpsAction>>,
    ) -> Self {
        Self {
            inner,
            index,
            graph,
        }
    }
}
impl Node for RpsNode {
    fn bucket(&self) -> RpsBucket {
        RpsBucket {}
    }
    fn player(&self) -> &RpsPlayer {
        self.inner().player()
    }
    fn parent(&self) -> Option<&Self> {
        self.graph()
            .neighbors_directed(self.index, Incoming)
            .next()
            .map(|p| {
                self.graph()
                    .node_weight(p)
                    .expect("tree property: if incoming edge, then parent")
            })
    }
    fn children(&self) -> Vec<&Self> {
        self.graph()
            .neighbors_directed(self.index, Outgoing)
            .map(|c| {
                self.graph()
                    .node_weight(c)
                    .expect("tree property: if outgoing edge, then child")
            })
            .collect()
    }
    fn incoming(&self) -> Option<&RpsAction> {
        self.graph()
            .edges_directed(self.index, Incoming)
            .next()
            .map(|e| e.weight())
    }
    fn outgoing(&self) -> Vec<&RpsAction> {
        self.graph()
            .edges_directed(self.index, Outgoing)
            .map(|e| e.weight())
            .collect()
    }
    fn utility(&self, player: &RpsPlayer) -> Utility {
        const R_WIN: Utility = 1.0;
        const P_WIN: Utility = 1.0;
        const S_WIN: Utility = 1.0; // we can modify payoffs to verify convergence
        let a1 = self
            .incoming()
            .expect("eval at terminal node, depth > 1")
            .turn();
        let a2 = self
            .parent()
            .expect("eval at terminal node, depth = 2")
            .incoming()
            .expect("eval at terminal node, depth = 2")
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

    type NPlayer = RpsPlayer;
    type NAction = RpsAction;
    type NBucket = RpsBucket;
}
