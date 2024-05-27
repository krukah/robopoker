use super::{action::RPSEdge, node::RPSNode, player::RPSPlayer, policy::RPSPolicy};
use crate::cfr::training::strategy::Strategy;
use std::collections::HashMap;

/// tabular Node > Policy
pub(crate) struct RPSStrategy<'tree> {
    pub policies: HashMap<RPSNode<'tree>, RPSPolicy>,
}

impl<'t> Strategy for RPSStrategy<'t> {
    type SPlayer = RPSPlayer;
    type SAction = RPSEdge;
    type SPolicy = RPSPolicy;
    type SNode = RPSNode<'t>;
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.policies
            .get(node)
            .expect("no policy stored for node, is this a new tree?")
    }
}
