use super::player::RPSPlayer;
use crate::cfr::training::marker::action::Action;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) enum Move {
    R,
    P,
    S,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct RPSEdge {
    player: RPSPlayer,
    turn: Move,
}

impl RPSEdge {
    pub(crate) fn new(player: RPSPlayer, turn: Move) -> Self {
        Self { player, turn }
    }

    pub(crate) fn turn(&self) -> Move {
        self.turn
    }
}

impl Hash for RPSEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.turn.hash(state);
    }
}

impl Action for RPSEdge {}
