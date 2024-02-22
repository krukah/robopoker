use super::seat::Seat;
use crate::cards::card::Card;

pub trait Actor {
    fn act(&self) -> Action;
}

pub enum Action<'a> {
    Draw(Card),
    Check(&'a mut Seat),
    Raise(&'a mut Seat, u32),
    Shove(&'a mut Seat, u32),
    Open(&'a mut Seat, u32),
    Call(&'a mut Seat, u32),
    Fold(&'a mut Seat),
}
