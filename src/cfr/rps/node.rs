use petgraph::graph::DiGraph;
use petgraph::graph::EdgeIndex;
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;

use super::bucket::RpsBucket;
use super::tree::RpsTree;
use crate::cfr::rps::action::{Move, RpsAction};
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::Utility;

/// Shared-lifetime game tree nodes
pub(crate) struct RpsNode<'t> {
    idx: NodeIndex,
    tree: &'t RpsTree<'t>,
    inner: &'t RpsPlayer,
}

impl<'t> RpsNode<'t> {}

impl Node for RpsNode<'_> {
    type NPlayer = RpsPlayer;
    type NAction = RpsAction;
    type NBucket = RpsBucket;

    fn utility(&self, player: &Self::NPlayer) -> Utility {
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
    fn bucket(&self) -> Self::NBucket {
        RpsBucket {}
    }
    fn player(&self) -> &Self::NPlayer {
        self.inner
    }
    fn parent(&self) -> Option<&Self> {
        todo!() // self.parent.map(|i| self.tree.peek(i))
    }
    fn incoming(&self) -> Option<&Self::NAction> {
        self.tree
            .graph()
            .edges_directed(self.idx, Incoming)
            .next()
            .map(|e| e.weight())
    }
    fn children(&self) -> Vec<&Self> {
        self.tree
            .graph()
            .edges_directed(self.idx, Outgoing)
            .map(|e| e.target())
            .map(|i| {
                self.tree
                    .graph()
                    .node_weight(i)
                    .expect("follwed directed edge downward in tree to node")
            })
            .collect()
    }
    fn outgoing(&self) -> Vec<&Self::NAction> {
        self.tree
            .graph()
            .edges_directed(self.idx, Outgoing)
            .map(|e| e.weight())
            .collect()
    }
}
