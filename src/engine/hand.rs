use super::{action::Action, node::Node};

pub struct Hand {
    pub node: Node,
    pub history: Vec<Action>,
    pub bblind: u32,
    pub sblind: u32,
}

impl Hand {}
