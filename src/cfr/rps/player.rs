// rps/player.rs
use crate::cfr::training::player::Player;

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum RPSPlayer {
    P1,
    P2,
}

impl Player for RPSPlayer {}
