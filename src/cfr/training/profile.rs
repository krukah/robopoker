use super::{
    action::Action, info::Info, node::Node, player::Player, policy::Policy, strategy::Strategy,
    Probability, Utility,
};

/// A profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
pub(crate) trait Profile {
    // required
    fn improve(&self, info: &Self::PInfo) -> Self::PPolicy;
    fn strategy(&self, player: &Self::PPlayer) -> &Self::PStrategy;
    fn running_regret(&self, info: &Self::PInfo, action: &Self::PAction) -> Utility;
    fn instant_regret(&self, info: &Self::PInfo, action: &Self::PAction) -> Utility;
    fn update_regret(&mut self, info: &Self::PInfo);
    fn update_policy(&mut self, info: &Self::PInfo);

    // provided
    // utility calculations
    fn regret(&self, info: &Self::PInfo, action: &Self::PAction) -> Utility {
        self.running_regret(info, action) + self.instant_regret(info, action)
    }
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
        self.strategy(node.player()).policy(node).weights(action)
    }
    fn cfactual_reach(&self, node: &Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.cfactual_reach(parent)
                    * if node.player() == parent.player() {
                        1.0
                    } else {
                        self.weight(parent, node.precedent().unwrap())
                    }
            }
        }
    }
    fn expected_reach(&self, node: &Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.expected_reach(parent) * self.weight(parent, node.precedent().unwrap())
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
    type PAction: Action<APlayer = Self::PPlayer>;
    type PPolicy: Policy<PAction = Self::PAction>;
    type PNode: Node<NAction = Self::PAction> + Node<NPlayer = Self::PPlayer>;
    type PInfo: Info<INode = Self::PNode, IAction = Self::PAction, IPlayer = Self::PPlayer>;
    type PStrategy: Strategy
        + Strategy<SNode = Self::PNode>
        + Strategy<SPolicy = Self::PPolicy>
        + Strategy<SPlayer = Self::PPlayer>
        + Strategy<SAction = Self::PAction>;
}
