use super::seat::Seat;
use crate::cards::card::Card;
use std::cell::RefCell;

pub trait Actor {
    fn act(&self) -> Action;
}

pub enum Action {
    Draw(Card),
    Fold(RefCell<Seat>),
    Check(RefCell<Seat>),
    Call(RefCell<Seat>, u32),
    Open(RefCell<Seat>, u32),
    Raise(RefCell<Seat>, u32),
    Shove(RefCell<Seat>, u32),
}
