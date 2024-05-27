use super::{
    action::Action, info::Info, node::Node, player::Player, policy::Policy, profile::Profile,
    strategy::Strategy, tree::Tree, Probability, Utility,
};

pub(crate) trait Optimizer {
    fn update_regret(&mut self, info: &Self::OInfo);
    fn update_policy(&mut self, info: &Self::OInfo);

    fn this_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn last_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn next_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn next_policy(&self, info: &Self::OInfo) -> Self::OPolicy;

    fn regret_vector(&self, info: &Self::OInfo) -> Vec<Utility>;
    fn policy_vector(&self, info: &Self::OInfo) -> Vec<Probability>;

    type OPlayer: Player;
    type OAction: Action<APlayer = Self::OPlayer>;
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
