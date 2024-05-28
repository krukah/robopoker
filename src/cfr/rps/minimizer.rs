use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::signal::RpsSignal;
use crate::cfr::rps::tree::RpsTree;
use crate::cfr::training::learning::minimizer::Minimizer;
use crate::cfr::training::tree::info::Info;
use crate::cfr::training::tree::tree::Tree;
use crate::cfr::training::Probability;
use crate::cfr::training::Utility;
use std::collections::HashMap;

pub(crate) struct RpsMinimizer {
    regrets: HashMap<RpsSignal, HashMap<RpsEdge, Utility>>,
    profile: HashMap<RpsSignal, HashMap<RpsEdge, Probability>>,
}

impl RpsMinimizer {
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
            let signal = info.signal();
            for action in info.available() {
                self.profile
                    .entry(signal)
                    .or_insert_with(HashMap::new)
                    .insert(**action, weight);
                self.regrets
                    .entry(signal)
                    .or_insert_with(HashMap::new)
                    .insert(**action, regret);
            }
        }
    }
}

impl Minimizer for RpsMinimizer {
    fn profile(&self) -> &Self::OProfile {
        &self.profile
    }
    fn policy_vector(&self, info: &Self::OInfo) -> Vec<(Self::OAction, Probability)> {
        let regrets = info
            .available()
            .iter()
            .map(|action| (**action, self.running_regret(info, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(Self::OAction, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    fn running_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        *self
            .regrets
            .get(&info.signal())
            .expect("regret initialized for infoset")
            .get(action)
            .expect("regret initialized for actions")
    }

    fn update_policy(&mut self, info: &Self::OInfo) {
        for (ref action, weight) in self.policy_vector(info) {
            *self
                .profile
                .get_mut(&info.signal())
                .expect("weight initialized for infoset")
                .get_mut(action)
                .expect("weight initialized for actions") = weight;
        }
    }
    fn update_regret(&mut self, info: &Self::OInfo) {
        for (ref action, regret) in self.regret_vector(info) {
            *self
                .regrets
                .get_mut(&info.signal())
                .expect("regret initialized for infoset")
                .get_mut(action)
                .expect("regret initialized for actions") += regret;
        }
    }

    type OPlayer = RpsPlayer;
    type OAction = RpsEdge;
    type OTree = RpsTree<'static>;
    type ONode = RpsNode<'static>;
    type OInfo = RpsInfo<'static>;
    type OPolicy = HashMap<RpsEdge, Probability>;
    type OProfile = HashMap<RpsSignal, HashMap<RpsEdge, Probability>>;
    type OStrategy = HashMap<RpsSignal, HashMap<RpsEdge, Probability>>;
}
