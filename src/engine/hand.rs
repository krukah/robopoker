use super::{action::Action, node::Node};

pub struct Hand {
    pub node: Node,
    pub history: Vec<Action>,
    pub bblind: u32,
    pub sblind: u32,
}

impl Hand {
    pub fn new() -> Hand {
        Hand {
            node: Node::new(),
            history: Vec::new(),
            bblind: 2,
            sblind: 1,
        }
    }

    pub fn reset(&mut self) {
        self.node = Node::new(self.node.table.seats.clone());
        self.history.clear();
    }
}
