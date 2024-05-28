use crate::cfr::rps::action::RPSEdge;
use crate::cfr::rps::info::RPSInfo;
use crate::cfr::rps::node::RPSNode;
use crate::cfr::rps::player::RPSPlayer;
use crate::cfr::rps::policy::RPSPolicy;
use crate::cfr::rps::profile::RPSProfile;
use crate::cfr::rps::signal::RPSSignal;
use crate::cfr::rps::strategy::RPSStrategy;
use crate::cfr::rps::tree::RPSTree;
use crate::cfr::training::learning::minimizer::Minimizer;
use crate::cfr::training::learning::policy::Policy;
use crate::cfr::training::learning::profile::Profile;
use crate::cfr::training::learning::strategy::Strategy;
use crate::cfr::training::tree::info::Info;
use crate::cfr::training::tree::node::Node;
use crate::cfr::training::Probability;
use crate::cfr::training::Utility;
use std::collections::HashMap;

impl Profile for HashMap<RPSSignal, HashMap<RPSEdge, Probability>> {
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self
    }
    type PAction = RPSEdge;
    type PPlayer = RPSPlayer;
    type PPolicy = RPSPolicy;
    type PNode = RPSNode<'static>;
    type PInfo = RPSInfo<'static>;
    type PStrategy = RPSStrategy<'static>;
}

impl Strategy for HashMap<RPSSignal, HashMap<RPSEdge, Probability>> {
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy {
        self.get(node.sign())
            .expect("policy initialized across signature set")
    }
    type SPlayer = RPSPlayer;
    type SAction = RPSEdge;
    type SPolicy = RPSPolicy;
    type SNode = RPSNode<'static>;
}

impl Policy for HashMap<RPSEdge, Probability> {
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
    type PAction = RPSEdge;
}

pub(crate) struct RPSMinimizer {
    regrets: HashMap<RPSSignal, HashMap<RPSEdge, Utility>>,
    profile: HashMap<RPSSignal, HashMap<RPSEdge, Probability>>,
}

impl RPSMinimizer {
    pub fn new() -> Self {
        let profile = HashMap::new();
        let regrets = HashMap::new();
        Self { regrets, profile }
    }
    pub fn initialize(&mut self, tree: &RPSTree) {
        let mut profile = RPSProfile::new();
        let mut regrets = RPSRegrets::new();
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

    // fn next_regret(&self, info: &RPSInfo<'t>, action: &RPSEdge) -> Utility {
    //     self.last_regret(info, action) + self.this_regret(info, action)
    // }
    // fn last_regret(&self, info: &RPSInfo<'t>, action: &RPSEdge) -> Utility {
    //     self.regrets.get_regret(&info.sign(), action)
    // }
    // fn this_regret(&self, info: &RPSInfo<'t>, action: &RPSEdge) -> Utility {
    //     info.roots()
    //         .iter()
    //         .map(|root| self.profile.gain(root, action))
    //         .sum::<Utility>()
    // }

    // fn policy_vector(&self, info: &RPSInfo<'t>) -> Vec<Probability> {
    //     let regrets = self.regret_vector(info);
    //     let sum = regrets.iter().sum::<Utility>();
    //     regrets.iter().map(|regret| regret / sum).collect()
    // }
    // fn regret_vector(&self, info: &RPSInfo<'t>) -> Vec<Utility> {
    //      let regrets =   info.available()
    //         .iter()
    //         .map(|action| self.last_regret(info, action))
    //         .map(|regret| regret.max(Utility::MIN_POSITIVE))
    //         .collect::<Vec<Probability>>();
    //     let sum = regrets.iter().sum::<Utility>();
    // }
}

impl Minimizer for RPSMinimizer {
    fn profile(&self) -> &Self::OProfile {
        &self.profile
    }
    fn update_policy(&mut self, info: &Self::OInfo) {
        for (action, weight) in info
            .available()
            .iter()
            .map(|action| **action)
            .zip(self.policy_vector(info).into_iter())
            .collect::<Vec<(RPSEdge, Probability)>>()
        {
            self.profile.update_weight(info.signal(), action, weight)
        }
    }
    fn update_regret(&mut self, info: &Self::OInfo) {
        for (action, regret) in info
            .available()
            .iter()
            .map(|action| (**action, self.next_regret(info, action)))
            .collect::<Vec<(RPSEdge, Utility)>>()
        {
            self.regrets.update_regret(&info.signal(), &action, regret);
        }
    }

    type OPlayer = RPSPlayer;
    type OAction = RPSEdge;
    type OPolicy = RPSPolicy;
    type OTree = RPSTree<'static>;
    type ONode = RPSNode<'static>;
    type OInfo = RPSInfo<'static>;
    type OProfile = RPSProfile<'static>;
    type OStrategy = RPSStrategy<'static>;
}
