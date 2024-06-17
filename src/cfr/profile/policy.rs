use crate::cfr::tree::rps::action::Edge;
use crate::Probability;
use std::collections::HashMap;

pub struct Policy(pub HashMap<Edge, Probability>);

impl Policy {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
