pub trait Player: Debug {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action;
}
use super::{action::Action, hand::Hand, seat::Seat};
use std::fmt::Debug;
