use crate::cfr::traits::learning::policy::Policy;
use crate::cfr::traits::learning::profile::Profile;
use crate::cfr::traits::learning::strategy::Strategy;
use crate::cfr::traits::marker::action::Action;
use crate::cfr::traits::marker::player::Player;
use crate::cfr::traits::tree::info::Info;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::tree::tree::Tree;
use crate::cfr::traits::{Probability, Utility};

pub(crate) trait Optimizer {
    fn profile(&self) -> &Self::OProfile;

    fn update_regret(&mut self, info: &Self::OInfo);
    fn update_policy(&mut self, info: &Self::OInfo);

    fn current_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn instant_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.profile().gain(root, action))
            .sum::<Utility>()
    }
    fn pending_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        self.instant_regret(info, action) + self.current_regret(info, action)
    }

    fn policy_vector(&self, info: &Self::OInfo) -> Vec<(Self::OAction, Probability)> {
        let regrets = info
            .available()
            .iter()
            .map(|action| (**action, self.current_regret(info, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(Self::OAction, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
        // uses RegretMatching+ to compute policy from current regrets
    }
    fn regret_vector(&self, info: &Self::OInfo) -> Vec<(Self::OAction, Utility)> {
        info.available()
            .iter()
            .map(|action| (**action, self.pending_regret(info, action)))
            .collect()
    }

    type OPlayer: Player;
    type OAction: Action;
    type OPolicy: Policy<PAction = Self::OAction>;
    type ONode: Node<NAction = Self::OAction> + Node<NPlayer = Self::OPlayer>;
    type OInfo: Info
        + Info<INode = Self::ONode>
        + Info<IAction = Self::OAction>
        + Info<IPlayer = Self::OPlayer>;
    type OTree: Tree
        + Tree<TInfo = Self::OInfo>
        + Tree<TNode = Self::ONode>
        + Tree<TEdge = Self::OAction>
        + Tree<TPlayer = Self::OPlayer>;
    type OStrategy: Strategy
        + Strategy<SNode = Self::ONode>
        + Strategy<SAction = Self::OAction>
        + Strategy<SPlayer = Self::OPlayer>
        + Strategy<SPolicy = Self::OPolicy>;
    type OProfile: Profile
        + Profile<PStrategy = Self::OStrategy>
        + Profile<PInfo = Self::OInfo>
        + Profile<PNode = Self::ONode>
        + Profile<PAction = Self::OAction>
        + Profile<PPolicy = Self::OPolicy>
        + Profile<PPlayer = Self::OPlayer>;
}
