use super::player::Player;
use crate::cards::board::Street;
use std::cell::RefCell;

pub trait Actor {
    fn act(&self) -> Action;
}

pub enum Action {
    Draw(Street),
    Fold(RefCell<Player>),
    Check(RefCell<Player>),
    Call(RefCell<Player>, u32),
    Open(RefCell<Player>, u32),
    Raise(RefCell<Player>, u32),
    Shove(RefCell<Player>, u32),
}
