use super::signal::RpsSignal;
use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::training::learning::strategy::Strategy;
use crate::cfr::training::tree::node::Node;
use crate::cfr::training::Probability;
use std::collections::HashMap;

impl Strategy for HashMap<RpsSignal, HashMap<RpsEdge, Probability>> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.get(&node.signal())
            .expect("policy initialized across signature set")
    }
    type SNode = RpsNode<'static>;
    type SPlayer = RpsPlayer;
    type SAction = RpsEdge;
    type SPolicy = HashMap<RpsEdge, Probability>;
}
