use super::{
    action::Action, info::Info, node::Node, player::Player, policy::Policy, profile::Profile,
    strategy::Strategy, tree::Tree,
};

/// A Trainer will take a Profile and a Tree and iteratively consume/replace a new Profile on each iteration. Implementations may include RegretMatching+, Linear RM, Discounted RM, Parametrized RM, etc.
pub(crate) trait Trainer {
    // required
    fn train(&mut self, n: usize);
    fn save(&self);
    // fn tree(&self) -> Self::TTree;
    // fn profile(&self) -> &Self::TProfile;

    type TPlayer: Player;
    type TAction: Action<APlayer = Self::TPlayer>;
    type TPolicy: Policy<PAction = Self::TAction>;
    type TNode: Node<NAction = Self::TAction> + Node<NPlayer = Self::TPlayer>;
    type TInfo: Info
        + Info<INode = Self::TNode>
        + Info<IAction = Self::TAction>
        + Info<IPlayer = Self::TPlayer>;
    type TTree: Tree
        + Tree<TInfo = Self::TInfo>
        + Tree<TNode = Self::TNode>
        + Tree<TEdge = Self::TAction>
        + Tree<TPlayer = Self::TPlayer>;
    type TStrategy: Strategy
        + Strategy<SNode = Self::TNode>
        + Strategy<SAction = Self::TAction>
        + Strategy<SPlayer = Self::TPlayer>
        + Strategy<SPolicy = Self::TPolicy>;
    type TProfile: Profile
        + Profile<PStrategy = Self::TStrategy>
        + Profile<PInfo = Self::TInfo>
        + Profile<PNode = Self::TNode>
        + Profile<PAction = Self::TAction>
        + Profile<PPolicy = Self::TPolicy>
        + Profile<PPlayer = Self::TPlayer>;
}
