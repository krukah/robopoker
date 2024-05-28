use crate::cfr::training::marker::action::Action;
use crate::cfr::training::marker::player::Player;
use crate::cfr::training::tree::node::Node;
use std::hash::Hash;

/// A set of indistinguishable nodes compatible with the player's information, up to any abstraction. Intuitively, this is the support of the distribution over information unknown to the player whose turn to act.
pub(crate) trait Info: Eq + Hash {
    // required
    fn roots(&self) -> &Vec<&Self::INode>;

    // provided
    fn available(&self) -> &Vec<&Self::IAction> {
        self.roots().iter().next().unwrap().available()
    }

    type IPlayer: Player;
    type IAction: Action;
    type INode: Node<NAction = Self::IAction> + Node<NPlayer = Self::IPlayer>;
}
