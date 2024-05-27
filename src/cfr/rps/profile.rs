use super::{
    action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer, policy::RPSPolicy,
    strategy::RPSStrategy, tree::RPSTree,
};
use crate::cfr::training::{profile::Profile, tree::Tree};

/// constant Player > Strategy
pub(crate) struct RPSProfile<'tree> {
    pub strategy: RPSStrategy<'tree>,
}

impl<'t> RPSProfile<'t> {
    pub fn new(tree: &RPSTree) -> Self {
        for _ in tree.infos() {}
        todo!("initialize regrets & strategies for info")
    }
}

impl<'t> Profile for RPSProfile<'t> {
    type PPlayer = RPSPlayer;
    type PAction = RPSEdge;
    type PPolicy = RPSPolicy;
    type PNode = RPSNode<'t>;
    type PInfo = RPSInfo<'t>;
    type PStrategy = RPSStrategy<'t>;
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self.strategy
    }
}
