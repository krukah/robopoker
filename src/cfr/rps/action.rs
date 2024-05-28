use super::player::RpsPlayer;
use crate::cfr::training::marker::action::Action;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) enum Move {
    R,
    P,
    S,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct RpsEdge {
    player: RpsPlayer,
    turn: Move,
}

impl RpsEdge {
    pub(crate) fn new(player: RpsPlayer, turn: Move) -> Self {
        Self { player, turn }
    }

    pub(crate) fn turn(&self) -> Move {
        self.turn
    }
}

impl Hash for RpsEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.turn.hash(state);
    }
}

impl Action for RpsEdge {}
