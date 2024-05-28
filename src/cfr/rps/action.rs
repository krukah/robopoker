use super::player::RpsPlayer;
use crate::cfr::traits::marker::action::Action;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct RpsAction {
    player: RpsPlayer,
    turn: Move,
}

impl RpsAction {
    pub(crate) fn turn(&self) -> Move {
        self.turn
    }
}

impl Hash for RpsAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.turn.hash(state);
    }
}

impl Action for RpsAction {}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) enum Move {
    R,
    P,
    S,
}
