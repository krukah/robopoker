use crate::cfr::rps::action::RPSEdge;
use crate::cfr::rps::node::RPSNode;
use crate::cfr::rps::player::RPSPlayer;
use crate::cfr::rps::policy::RPSPolicy;
use crate::cfr::training::learning::strategy::Strategy;
use std::collections::HashMap;

pub(crate) struct RPSStrategy<'tree> {
    policies: HashMap<RPSNode<'tree>, RPSPolicy>,
}

impl<'t> RPSStrategy<'t> {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }
}

impl<'t> Strategy for RPSStrategy<'t> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.policies
            .get(node)
            .expect("policy initialized across signature set")
    }

    type SPlayer = RPSPlayer;
    type SAction = RPSEdge;
    type SPolicy = RPSPolicy;
    type SNode = RPSNode<'t>;
}
