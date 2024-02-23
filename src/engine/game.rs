#[derive(Debug)]
pub struct Game {
    pub bblind: u32,
    pub sblind: u32,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub actions: Vec<Action>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            sblind: 1,
            bblind: 2,
            tail: Node::new(),
            head: Node::new(),
            actions: Vec::new(),
        }
    }
}
use super::{action::Action, node::Node};
