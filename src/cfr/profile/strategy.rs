use crate::cfr::traits::action::Edge;
use crate::Probability;
use std::collections::HashMap;

pub(crate) struct Policy(pub HashMap<Edge, Probability>);

impl Policy {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
