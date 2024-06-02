use super::bucket::RpsBucket;
use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::training::strategy::Strategy;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::Probability;
use std::collections::HashMap;

impl Strategy for HashMap<RpsBucket, HashMap<RpsAction, Probability>> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.get(&node.bucket())
            .expect("policy initialized across signature set")
    }
    type SNode = RpsNode;
    type SPlayer = RpsPlayer;
    type SAction = RpsAction;
    type SPolicy = HashMap<RpsAction, Probability>;
}
