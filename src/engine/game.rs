#[derive(Debug)]
pub struct Game {
    pub bblind: u32,
    pub sblind: u32,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub actions: Vec<Action>,
}

impl Game {
    pub fn new(seats: Vec<Seat>) -> Game {
        Game {
            sblind: 1,
            bblind: 2,
            tail: Node::new(seats.clone()),
            head: Node::new(seats),
            actions: Vec::new(),
        }
    }
}
use super::{action::Action, node::Node, seat::Seat};
