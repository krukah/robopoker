pub struct GameHand<'a> {
    pub tail: Node, // is this useful?
    pub head: Node,
    pub deck: Deck,
    pub history: Vec<Action<'a>>,
    pub bblind: u32,
    pub sblind: u32,
}

impl<'a> GameHand<'a> {
    pub fn new(seats: Vec<Seat>) -> GameHand<'a> {
        GameHand {
            sblind: 1,
            bblind: 2,
            tail: Node::new(seats.clone()),
            head: Node::new(seats),
            deck: Deck::new(),
            history: Vec::new(),
        }
    }
}
use super::{action::Action, node::Node, seat::Seat};
use crate::cards::deck::Deck;
