// rps/player.rs
use crate::cfr::traits::marker::player::Player;

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum RpsPlayer {
    P1,
    P2,
}

impl Player for RpsPlayer {}
