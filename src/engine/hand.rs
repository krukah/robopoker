pub struct Hand {
    pub tail: Node,
    pub head: Node,
    pub deck: Deck,
    pub history: Vec<Action>,
    pub bblind: u32,
    pub sblind: u32,
}

impl Hand {
    pub fn new(players: Vec<&Player>) -> Hand {
        let node = Node::new();
        Hand {
            tail: node,
            head: node.clone(),
            deck: Deck::new(),
            history: Vec::new(),
            bblind: 2,
            sblind: 1,
        }
    }
}
use crate::cards::deck::Deck;

use super::{action::Action, node::Node, player::Player, seat::Seat};
