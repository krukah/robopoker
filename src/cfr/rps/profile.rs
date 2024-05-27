use super::{
    action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer, policy::RPSPolicy,
    strategy::RPSStrategy, tree::RPSTree,
};
use crate::cfr::training::{info::Info, profile::Profile, tree::Tree, Probability, Utility};
use std::collections::HashMap;

/// constant Player > Strategy
///
type RPSRegrets<'tree> = HashMap<&'tree RPSInfo<'tree>, HashMap<&'tree RPSEdge, Utility>>;
pub(crate) struct RPSProfile<'tree> {
    pub regrets: RPSRegrets<'tree>, // info -> action -> utility
    pub strategy: RPSStrategy<'tree>,
}

impl<'t> RPSProfile<'t> {
    pub fn new() -> Self {
        todo!("allocate empty HashMaps for regrets & strategy")
    }
    pub fn walk(&mut self, tree: &RPSTree) {
        for _ in tree.infos() {
            todo!("initialize regrets & strategies for info");
        }
    }

    fn policy_vector(&self, info: &RPSInfo) -> Vec<Probability> {
        let regrets = self.regret_vector(info);
        let sum = regrets.iter().sum::<Utility>();
        regrets.iter().map(|regret| regret / sum).collect()
    }
    fn regret_vector(&self, info: &RPSInfo) -> Vec<Utility> {
        info.available()
            .iter()
            .map(|action| self.regret(info, action))
            .map(|regret| regret.max(Utility::MIN_POSITIVE))
            .collect()
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
    fn improve(&self, info: &Self::PInfo) -> Self::PPolicy {
        Self::PPolicy::new(
            info.available()
                .iter()
                .map(|action| **action)
                .zip(self.policy_vector(info).into_iter())
                .collect::<HashMap<Self::PAction, Probability>>(),
        )
    }
    fn running_regret(&self, info: &Self::PInfo, action: &Self::PAction) -> Utility {
        *self
            .regrets
            .get(info)
            .expect("no regrets stored for info, is this a new tree?")
            .get(action)
            .expect("no regret stored for action, is this a new tree?")
    }
    fn instant_regret(&self, info: &Self::PInfo, action: &Self::PAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.gain(root, action)) //? self.profile().regret(info, action)
            .sum::<Utility>()
    }
    fn update_regret(&mut self, info: &Self::PInfo) {
        let actions = info
            .available()
            .iter()
            .map(|a| **a)
            .collect::<Vec<RPSEdge>>();
        for action in actions.iter() {
            let regret = self.regret(info, action);
            let value = self
                .regrets
                .get_mut(info)
                .expect("regret initialized")
                .get_mut(action)
                .expect("regret initialized");
            *value = regret;
        }
    }
    fn update_policy(&mut self, info: &Self::PInfo) {
        let policy = self.improve(info);
        let signature = todo!("abstraction for info<>node to agree upon");
        self.strategy.policies.insert(signature, policy);
    }
}
