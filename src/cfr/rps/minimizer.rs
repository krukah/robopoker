use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::policy::RpsPolicy;
use crate::cfr::rps::profile::RpsProfile;
use crate::cfr::rps::signal::RpsSignal;
use crate::cfr::rps::strategy::RpsStrategy;
use crate::cfr::rps::tree::RpsTree;
use crate::cfr::training::learning::minimizer::Minimizer;
use crate::cfr::training::learning::policy::Policy;
use crate::cfr::training::learning::profile::Profile;
use crate::cfr::training::learning::strategy::Strategy;
use crate::cfr::training::tree::info::Info;
use crate::cfr::training::tree::node::Node;
use crate::cfr::training::Probability;
use crate::cfr::training::Utility;
use std::collections::HashMap;

impl Profile for HashMap<RpsSignal, HashMap<RpsEdge, Probability>> {
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self
    }
    type PAction = RpsEdge;
    type PPlayer = RpsPlayer;
    type PPolicy = RpsPolicy;
    type PNode = RpsNode<'static>;
    type PInfo = RpsInfo<'static>;
    type PStrategy = RpsStrategy<'static>;
}

impl Strategy for HashMap<RpsSignal, HashMap<RpsEdge, Probability>> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.get(node.sign())
            .expect("policy initialized across signature set")
    }
    type SPlayer = RpsPlayer;
    type SAction = RpsEdge;
    type SPolicy = RpsPolicy;
    type SNode = RpsNode<'static>;
}

impl Policy for HashMap<RpsEdge, Probability> {
    fn weight(&self, action: &Self::PAction) -> Probability {
        self.get(action)
            .expect("weight initialized across action set")
    }
    fn sample(&self) -> &Self::PAction {
        self.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0
    }
    type PAction = RpsEdge;
}

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
    pub fn initialize(&mut self, tree: &RpsTree) {
        let mut profile = RpsProfile::new();
        let mut regrets = RpsRegrets::new();
        for info in tree.infos() {
            let n = info.available().len();
            let weight = 1.0 / n as Probability;
            let regret = 0.0;
            for action in info.available() {
                profile.set_weight(&info.sign(), *action, weight);
                regrets.set_regret(&info.sign(), *action, regret);
            }
        }
    }

    // fn next_regret(&self, info: &RpsInfo<'t>, action: &RpsEdge) -> Utility {
    //     self.last_regret(info, action) + self.this_regret(info, action)
    // }
    // fn last_regret(&self, info: &RpsInfo<'t>, action: &RpsEdge) -> Utility {
    //     self.regrets.get_regret(&info.sign(), action)
    // }
    // fn this_regret(&self, info: &RpsInfo<'t>, action: &RpsEdge) -> Utility {
    //     info.roots()
    //         .iter()
    //         .map(|root| self.profile.gain(root, action))
    //         .sum::<Utility>()
    // }

    // fn policy_vector(&self, info: &RpsInfo<'t>) -> Vec<Probability> {
    //     let regrets = self.regret_vector(info);
    //     let sum = regrets.iter().sum::<Utility>();
    //     regrets.iter().map(|regret| regret / sum).collect()
    // }
    // fn regret_vector(&self, info: &RpsInfo<'t>) -> Vec<Utility> {
    //      let regrets =   info.available()
    //         .iter()
    //         .map(|action| self.last_regret(info, action))
    //         .map(|regret| regret.max(Utility::MIN_POSITIVE))
    //         .collect::<Vec<Probability>>();
    //     let sum = regrets.iter().sum::<Utility>();
    // }
}

impl Minimizer for RpsMinimizer {
    fn profile(&self) -> &Self::OProfile {
        &self.profile
    }
    fn update_policy(&mut self, info: &Self::OInfo) {
        for (action, weight) in info
            .available()
            .iter()
            .map(|action| **action)
            .zip(self.policy_vector(info).into_iter())
            .collect::<Vec<(RpsEdge, Probability)>>()
        {
            self.profile.update_weight(info.signal(), action, weight)
        }
    }
    fn update_regret(&mut self, info: &Self::OInfo) {
        for (action, regret) in info
            .available()
            .iter()
            .map(|action| (**action, self.next_regret(info, action)))
            .collect::<Vec<(RpsEdge, Utility)>>()
        {
            self.regrets.update_regret(&info.signal(), &action, regret);
        }
    }

    type OPlayer = RpsPlayer;
    type OAction = RpsEdge;
    type OPolicy = RpsPolicy;
    type OTree = RpsTree<'static>;
    type ONode = RpsNode<'static>;
    type OInfo = RpsInfo<'static>;
    type OProfile = RpsProfile<'static>;
    type OStrategy = RpsStrategy<'static>;
}
