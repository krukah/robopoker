use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::policy::RpsPolicy;
use crate::cfr::training::learning::strategy::Strategy;
use std::collections::HashMap;

pub(crate) struct RpsStrategy<'tree> {
    policies: HashMap<RpsNode<'tree>, RpsPolicy>,
}

impl<'t> RpsStrategy<'t> {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }
}

impl<'t> Strategy for RpsStrategy<'t> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.policies
            .get(node)
            .expect("policy initialized across signature set")
    }

    type SPlayer = RpsPlayer;
    type SAction = RpsEdge;
    type SPolicy = RpsPolicy;
    type SNode = RpsNode<'t>;
}
