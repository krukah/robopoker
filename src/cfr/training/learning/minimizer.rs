use crate::cfr::training::learning::policy::Policy;
use crate::cfr::training::learning::profile::Profile;
use crate::cfr::training::learning::strategy::Strategy;
use crate::cfr::training::marker::action::Action;
use crate::cfr::training::marker::player::Player;
use crate::cfr::training::tree::info::Info;
use crate::cfr::training::tree::node::Node;
use crate::cfr::training::tree::tree::Tree;
use crate::cfr::training::{Probability, Utility};

pub(crate) trait Minimizer {
    fn profile(&self) -> &Self::OProfile;

    fn update_regret(&mut self, info: &Self::OInfo);
    fn update_policy(&mut self, info: &Self::OInfo);

    fn running_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn instant_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.profile().gain(root, action))
            .sum::<Utility>()
    }
    fn looming_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility {
        self.instant_regret(info, action) + self.running_regret(info, action)
    }

    fn policy_vector(&self, info: &Self::OInfo) -> Vec<(Self::OAction, Probability)>;
    fn regret_vector(&self, info: &Self::OInfo) -> Vec<(Self::OAction, Utility)> {
        info.available()
            .iter()
            .map(|action| (**action, self.looming_regret(info, action)))
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
