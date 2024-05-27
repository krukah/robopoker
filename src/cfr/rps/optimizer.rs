use super::{
    action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer, policy::RPSPolicy,
    profile::RPSProfile, strategy::RPSStrategy, tree::RPSTree,
};
use crate::cfr::training::{
    info::Info, optimizer::Optimizer, profile::Profile, Probability, Utility,
};
use std::collections::HashMap;

type RPSRegrets<'t> = HashMap<&'t RPSInfo<'t>, HashMap<&'t RPSEdge, Utility>>;

pub(crate) struct RPSOptimzer<'tree> {
    regrets: RPSRegrets<'tree>,
    profile: RPSProfile<'tree>,
}

impl RPSOptimzer<'_> {
    pub fn new(tree: &RPSTree) -> Self {
        let regrets = HashMap::new();
        let profile = RPSProfile::new(tree);
        Self { regrets, profile }
    }
}

impl<'t> Optimizer for RPSOptimzer<'t> {
    type OPlayer = RPSPlayer;
    type OAction = RPSEdge;
    type OPolicy = RPSPolicy;
    type OTree = RPSTree<'t>;
    type ONode = RPSNode<'t>;
    type OInfo = RPSInfo<'t>;
    type OProfile = RPSProfile<'t>;
    type OStrategy = RPSStrategy<'t>;

    fn update_regret(&mut self, info: &Self::OInfo) {
        let actions = info
            .available()
            .iter()
            .map(|a| **a)
            .collect::<Vec<RPSEdge>>();
        for action in actions.iter() {
            let regret = self.next_regret(info, action);
            let value = self
                .regrets
                .get_mut(info)
                .expect("regret initialized")
                .get_mut(action)
                .expect("regret initialized");
            *value = regret;
        }
    }
    fn update_policy(&mut self, info: &Self::OInfo) {
        let node = info.available().first().expect("no actions available");
        self.profile
            .strategy
            .policies
            .insert(node, self.next_policy(info));
        todo!("Info and Node need associated Signature to lookup/insert into Profile::Strategy::Policy")
    }

    fn next_policy(&self, info: &Self::OInfo) -> Self::OPolicy {
        Self::OPolicy::new(
            info.available()
                .iter()
                .map(|action| **action)
                .zip(self.policy_vector(info).into_iter())
                .collect::<HashMap<Self::OAction, Probability>>(),
        )
    }
    fn next_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        self.this_regret(info, action) + self.last_regret(info, action)
    }
    fn this_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        *self
            .regrets
            .get(info)
            .expect("no regrets stored for info, is this a new tree?")
            .get(action)
            .expect("no regret stored for action, is this a new tree?")
    }
    fn last_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.profile.gain(root, action))
            .sum::<Utility>()
    }

    fn regret_vector(&self, info: &Self::OInfo) -> Vec<Utility> {
        info.available()
            .iter()
            .map(|action| self.this_regret(info, action))
            .map(|regret| regret.max(Utility::MIN_POSITIVE))
            .collect()
    }
    fn policy_vector(&self, info: &Self::OInfo) -> Vec<Probability> {
        let regrets = self.regret_vector(info);
        let sum = regrets.iter().sum::<Utility>();
        regrets.iter().map(|regret| regret / sum).collect()
    }
}
