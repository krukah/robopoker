use crate::cfr::traits::marker::action::Action;
use crate::cfr::traits::marker::bucket::Bucket;
use crate::cfr::traits::marker::player::Player;
use crate::cfr::traits::tree::node::Node;

/// A set of indistinguishable nodes compatible with the player's information, up to any abstraction. Intuitively, this is the support of the distribution over information unknown to the player whose turn to act.
pub(crate) trait Info {
    // required
    fn roots(&self) -> &Vec<&Self::INode>;
    fn bucket(&self) -> Self::IBucket;

    // provided
    fn available(&self) -> Vec<&Self::IAction> {
        self.roots().iter().next().unwrap().child_edges()
    }

    type IPlayer: Player;
    type IAction: Action;
    type IBucket: Bucket;
    type INode: Node
        + Node<NAction = Self::IAction>
        + Node<NPlayer = Self::IPlayer>
        + Node<NBucket = Self::IBucket>;
}
