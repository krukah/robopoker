use super::{action::Action, node::Node};

pub struct Hand {
    pub node: Node,
    pub history: Vec<Action>,
}

impl Hand {}
