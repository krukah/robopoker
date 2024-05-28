use crate::cfr::traits::marker::action::Action;
use crate::cfr::traits::marker::player::Player;
use crate::cfr::traits::training::optimizer::Optimizer;
use crate::cfr::traits::training::policy::Policy;
use crate::cfr::traits::training::profile::Profile;
use crate::cfr::traits::training::strategy::Strategy;
use crate::cfr::traits::tree::info::Info;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::tree::tree::Tree;

/// A Trainer will take a Profile and a Tree and iteratively consume/replace a new Profile on each iteration. Implementations may include RegretMatching+, Linear RM, Discounted RM, Parametrized RM, etc.
pub(crate) trait Trainer {
    // required
    fn train(&mut self, n: usize);
    fn save(&self);

    type TPlayer: Player;
    type TAction: Action;
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
    type TMinimizer: Optimizer
        + Optimizer<OProfile = Self::TProfile>
        + Optimizer<OStrategy = Self::TStrategy>
        + Optimizer<OInfo = Self::TInfo>
        + Optimizer<ONode = Self::TNode>
        + Optimizer<OPolicy = Self::TPolicy>
        + Optimizer<OPlayer = Self::TPlayer>
        + Optimizer<OAction = Self::TAction>;
}
