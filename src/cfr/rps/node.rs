use super::{
    action::{Move, RPSEdge},
    player::RPSPlayer,
};
use crate::cfr::training::{node::Node, Utility};
use std::hash::{Hash, Hasher};

/// Shared-lifetime game tree nodes
#[derive(PartialEq, Eq)]
pub(crate) struct RPSNode<'tree> {
    player: &'tree RPSPlayer,
    parent: Option<&'tree RPSNode<'tree>>,
    precedent: Option<&'tree RPSEdge>,
    children: Vec<&'tree RPSNode<'tree>>,
    available: Vec<&'tree RPSEdge>,
}

impl Hash for RPSNode<'_> {
    /// lucky for us, every single node in RPS has the same abstraction lookup hash, which is to say there is no information to inform your decision.
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state)
    }
}

impl Node for RPSNode<'_> {
    type NPlayer = RPSPlayer;
    type NAction = RPSEdge;
    fn player(&self) -> &Self::NPlayer {
        self.player
    }
    fn available(&self) -> &Vec<&Self::NAction> {
        &self.available
    }
    fn children(&self) -> &Vec<&Self> {
        &self.children
    }
    fn parent(&self) -> &Option<&Self> {
        &self.parent
    }
    fn precedent(&self) -> &Option<&Self::NAction> {
        &self.precedent
    }
    fn utility(&self, player: &Self::NPlayer) -> Utility {
        const R_WIN: Utility = 1.0;
        const P_WIN: Utility = 1.0;
        const S_WIN: Utility = 1.0; // we can modify payoffs to verify convergence
        let a1 = self.precedent.expect("terminal node, depth = 2").action();
        let a2 = self
            .parent
            .expect("terminal node, depth = 2")
            .precedent
            .expect("terminal node, depth = 2")
            .action();
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
            RPSPlayer::P1 => 0.0 + 1.0,
            RPSPlayer::P2 => 0.0 - 1.0,
        };
        direction * payoff
    }
}
