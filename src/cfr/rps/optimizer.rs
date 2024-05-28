use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::bucket::RpsBucket;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::tree::RpsTree;
use crate::cfr::traits::training::optimizer::Optimizer;
use crate::cfr::traits::tree::info::Info;
use crate::cfr::traits::tree::tree::Tree;
use crate::cfr::traits::Probability;
use crate::cfr::traits::Utility;
use std::collections::HashMap;

pub(crate) struct RpsOptimizer {
    regrets: HashMap<RpsBucket, HashMap<RpsAction, Utility>>,
    profile: HashMap<RpsBucket, HashMap<RpsAction, Probability>>,
}

impl RpsOptimizer {
    pub fn new() -> Self {
        let profile = HashMap::new();
        let regrets = HashMap::new();
        Self { regrets, profile }
    }
    pub fn scan(&mut self, tree: &RpsTree) {
        for info in tree.infos() {
            let n = info.available().len();
            let weight = 1.0 / n as Probability;
            let regret = 0.0;
            let bucket = info.bucket();
            for action in info.available() {
                self.profile
                    .entry(bucket)
                    .or_insert_with(HashMap::new)
                    .insert(**action, weight);
                self.regrets
                    .entry(bucket)
                    .or_insert_with(HashMap::new)
                    .insert(**action, regret);
            }
        }
    }
}

impl Optimizer for RpsOptimizer {
    fn profile(&self) -> &Self::OProfile {
        &self.profile
    }
    fn current_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        *self
            .regrets
            .get(&info.bucket())
            .expect("regret initialized for infoset")
            .get(action)
            .expect("regret initialized for actions")
    }

    fn update_policy(&mut self, info: &Self::OInfo) {
        for (ref action, weight) in self.policy_vector(info) {
            *self
                .profile
                .get_mut(&info.bucket())
                .expect("weight initialized for infoset")
                .get_mut(action)
                .expect("weight initialized for actions") = weight;
        }
    }
    fn update_regret(&mut self, info: &Self::OInfo) {
        for (ref action, regret) in self.regret_vector(info) {
            *self
                .regrets
                .get_mut(&info.bucket())
                .expect("regret initialized for infoset")
                .get_mut(action)
                .expect("regret initialized for actions") += regret;
        }
    }

    type OPlayer = RpsPlayer;
    type OAction = RpsAction;
    type OTree = RpsTree<'static>;
    type ONode = RpsNode<'static>;
    type OInfo = RpsInfo<'static>;
    type OPolicy = HashMap<RpsAction, Probability>;
    type OProfile = HashMap<RpsBucket, HashMap<RpsAction, Probability>>;
    type OStrategy = HashMap<RpsBucket, HashMap<RpsAction, Probability>>;
}
