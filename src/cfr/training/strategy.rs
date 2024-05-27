use super::{action::Action, node::Node, player::Player, policy::Policy};

/// A strategy (σ: player -> policy) is a function that assigns a policy to each h ∈ H, and therefore Ii ∈ Ii. Easily implemented as a HashMap<Info, Policy>.
pub(crate) trait Strategy {
    // required
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy;

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<NAction = Self::SAction> + Node<NPlayer = Self::SPlayer>;
}
