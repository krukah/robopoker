use super::signal::RpsSignal;
use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::learning::strategy::Strategy;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::Probability;
use std::collections::HashMap;

impl Strategy for HashMap<RpsSignal, HashMap<RpsAction, Probability>> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.get(&node.signal())
            .expect("policy initialized across signature set")
    }
    type SNode = RpsNode<'static>;
    type SPlayer = RpsPlayer;
    type SAction = RpsAction;
    type SPolicy = HashMap<RpsAction, Probability>;
}
