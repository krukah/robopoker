use crate::cfr::traits::marker::action::Action;
use crate::cfr::traits::marker::player::Player;
use crate::cfr::traits::training::policy::Policy;
use crate::cfr::traits::training::strategy::Strategy;
use crate::cfr::traits::tree::info::Info;
use crate::cfr::traits::tree::node::Node;
use crate::cfr::traits::Probability;
use crate::cfr::traits::Utility;

/// A profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
pub(crate) trait Profile {
    // required
    fn strategy(&self, player: &Self::PPlayer) -> &Self::PStrategy;

    // provided
    // utility calculations
    fn gain(&self, root: &Self::PNode, action: &Self::PAction) -> Utility {
        self.cfactual_value(root, action) - self.expected_value(root)
    }
    fn cfactual_value(&self, root: &Self::PNode, action: &Self::PAction) -> Utility {
        self.cfactual_reach(root)
            * root //                                       suppose you're here on purpose, counterfactually
                .follow(action) //                          suppose you're here on purpose, counterfactually
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&self, root: &Self::PNode) -> Utility {
        self.expected_reach(root)
            * root
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&self, root: &Self::PNode, leaf: &Self::PNode) -> Utility {
        leaf.utility(root.player())
            * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
    }
    // probability calculations
    fn weight(&self, node: &Self::PNode, action: &Self::PAction) -> Probability {
        self.strategy(node.player()).policy(node).weight(action)
    }
    fn cfactual_reach(&self, node: &Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.cfactual_reach(parent)
                    * if node.player() == parent.player() {
                        1.0
                    } else {
                        self.weight(parent, node.parent_edge().unwrap())
                    }
            }
        }
    }
    fn expected_reach(&self, node: &Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.expected_reach(parent) * self.weight(parent, node.parent_edge().unwrap())
            }
        }
    }
    fn relative_reach(&self, root: &Self::PNode, leaf: &Self::PNode) -> Probability {
        //? gotta optimize out integration over shared ancestors that cancels out in this division. Node: Eq? Hash?
        self.expected_reach(leaf) / self.expected_reach(root)
    }
    fn sampling_reach(&self, _oot: &Self::PNode, _eaf: &Self::PNode) -> Probability {
        1.0
    }

    type PPlayer: Player;
    type PAction: Action;
    type PPolicy: Policy<PAction = Self::PAction>;
    type PNode: Node<NAction = Self::PAction> + Node<NPlayer = Self::PPlayer>;
    type PInfo: Info<INode = Self::PNode, IAction = Self::PAction, IPlayer = Self::PPlayer>;
    type PStrategy: Strategy
        + Strategy<SNode = Self::PNode>
        + Strategy<SPolicy = Self::PPolicy>
        + Strategy<SPlayer = Self::PPlayer>
        + Strategy<SAction = Self::PAction>;
}
