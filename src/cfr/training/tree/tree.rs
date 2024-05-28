use crate::cfr::training::marker::action::Action;
use crate::cfr::training::marker::player::Player;
use crate::cfr::training::tree::info::Info;
use crate::cfr::training::tree::node::Node;

/// The owner all the Nodes, Actions, and Players in the context of a Solution. It also constrains the lifetime of references returned by its owned types. A vanilla implementation should build the full tree for small games. Monte Carlo implementations may sample paths conditional on given Profile, Trainer, or other constraints. The only contract is that the Tree must be able to partition decision nodes into Info sets.
pub(crate) trait Tree {
    // required
    fn infos(&self) -> Vec<&Self::TInfo>;

    type TPlayer: Player;
    type TEdge: Action;
    type TNode: Node<NAction = Self::TEdge> + Node<NPlayer = Self::TPlayer>;
    type TInfo: Info
        + Info<INode = Self::TNode>
        + Info<IAction = Self::TEdge>
        + Info<IPlayer = Self::TPlayer>;
}
