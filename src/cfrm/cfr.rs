#![allow(dead_code)]

/// Type alias encapsulates numberical precision for units of utility.
pub(crate) type Utility = f32;

/// Type alias encapsulates numberical precision for units of probability.
pub(crate) type Probability = f32;

/// An element of the finite set N of players, including chance.
pub(crate) trait Player: Eq {}

/// An element of the finite set of possible actions.
pub(crate) trait Action: Eq + Copy {
    // required
    fn player(&self) -> &Self::APlayer;

    type APlayer: Player;
}

/// A node,  history, game state, etc. Omnipotent, complete state of current game.
pub(crate) trait Node<'t> {
    // required
    fn parent(&'t self) -> Option<&'t Self>;
    fn precedent(&'t self) -> Option<&'t Self::NAction>;
    fn children(&'t self) -> &Vec<&'t Self>;
    fn available(&'t self) -> &Vec<&'t Self::NAction>;
    fn chooser(&'t self) -> &'t Self::NPlayer;
    fn utility(&'t self, player: &Self::NPlayer) -> Utility;

    // provided
    fn follow(&'t self, action: &Self::NAction) -> &'t Self {
        self.children()
            .iter()
            .find(|child| action == child.precedent().unwrap())
            .unwrap()
    }
    fn descendants(&'t self) -> Vec<&'t Self> {
        match self.children().len() {
            0 => vec![&self],
            _ => self
                .children()
                .iter()
                .map(|child| child.descendants())
                .flatten()
                .collect(),
        }
    }

    type NPlayer: Player;
    type NAction: Action<APlayer = Self::NPlayer>;
}

/// A set of indistinguishable nodes compatible with the player's information, up to any abstraction. Intuitively, this is the support of the distribution over information unknown to the player whose turn to act.
pub(crate) trait Info<'t> {
    // required
    fn roots(&'t self) -> &Vec<&'t Self::INode>;

    // provided
    fn endpoints(&'t self) -> Vec<&'t Self::INode> {
        self.roots()
            .iter()
            .map(|node| node.descendants())
            .flatten()
            .collect()
    }
    fn available(&'t self) -> &Vec<&'t Self::IAction> {
        self.roots().iter().next().unwrap().available()
    }

    type IPlayer: Player;
    type IAction: Action<APlayer = Self::IPlayer>;
    type INode: Node<'t, NAction = Self::IAction> + Node<'t, NPlayer = Self::IPlayer>;
}

/// The owner all the Nodes, Actions, and Players in the context of a Solution. It also constrains the lifetime of references returned by its owned types. A vanilla implementation should build the full tree for small games. Monte Carlo implementations may sample paths conditional on given Profile, Solver, or other constraints. The only contract is that the Tree must be able to partition decision nodes into Info sets.
pub(crate) trait Tree<'t> {
    // required
    fn infos(&'t self) -> &Vec<Self::TInfo>;

    type TPlayer: Player;
    type TEdge: Action<APlayer = Self::TPlayer>;
    type TNode: Node<'t, NAction = Self::TEdge> + Node<'t, NPlayer = Self::TPlayer>;
    type TInfo: Info<'t>
        + Info<'t, INode = Self::TNode>
        + Info<'t, IAction = Self::TEdge>
        + Info<'t, IPlayer = Self::TPlayer>;
}

/// A policy (P: node -> prob) is a distribution over A(Ii). Easily implemented as a HashMap<Aaction, Probability>.
pub(crate) trait Policy {
    // required
    fn weights(&self, action: &Self::PAction) -> Probability;

    type PAction: Action;
}

/// A strategy (σ: player -> policy) is a function that assigns a policy to each h ∈ H, and therefore Ii ∈ Ii. Easily implemented as a HashMap<Info, Policy>.
pub(crate) trait Strategy<'t> {
    // required
    fn policy(&'t self, node: &'t Self::SNode) -> &'t Self::SPolicy;

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<'t, NAction = Self::SAction> + Node<'t, NPlayer = Self::SPlayer>;
}

/// A profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
pub(crate) trait Profile<'t> {
    // required
    fn strategy(&'t self, player: &'t Self::PPlayer) -> &'t Self::PStrategy;

    // provided
    // utility calculations
    fn cfactual_value(&'t self, root: &'t Self::PNode, action: &'t Self::PAction) -> Utility {
        self.cfactual_reach(root)
            * root //                                       suppose you're here on purpose, counterfactually
                .follow(action) //                          suppose you're here on purpose, counterfactually
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&'t self, root: &'t Self::PNode) -> Utility {
        self.expected_reach(root)
            * root
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&'t self, root: &'t Self::PNode, leaf: &'t Self::PNode) -> Utility {
        leaf.utility(root.chooser())
            * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
    }
    // probability calculations
    fn weight(&'t self, node: &'t Self::PNode, action: &'t Self::PAction) -> Probability {
        self.strategy(node.chooser()).policy(node).weights(action)
    }
    fn cfactual_reach(&'t self, node: &'t Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.cfactual_reach(parent)
                    * if node.chooser() == parent.chooser() {
                        1.0
                    } else {
                        self.weight(parent, node.precedent().unwrap())
                    }
            }
        }
    }
    fn expected_reach(&'t self, node: &'t Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.expected_reach(parent) * self.weight(parent, node.precedent().unwrap())
            }
        }
    }
    fn relative_reach(&'t self, root: &'t Self::PNode, leaf: &'t Self::PNode) -> Probability {
        //? gotta optimize out integration over shared ancestors that cancels out in this division. Node: Eq? Hash?
        self.expected_reach(leaf) / self.expected_reach(root)
    }
    fn sampling_reach(&'t self, _oot: &'t Self::PNode, _eaf: &'t Self::PNode) -> Probability {
        1.0
    }

    type PPlayer: Player;
    type PAction: Action<APlayer = Self::PPlayer>;
    type PPolicy: Policy<PAction = Self::PAction>;
    type PNode: Node<'t, NAction = Self::PAction> + Node<'t, NPlayer = Self::PPlayer>;
    type PInfo: Info<'t>
        + Info<'t, INode = Self::PNode>
        + Info<'t, IAction = Self::PAction>
        + Info<'t, IPlayer = Self::PPlayer>;
    type PStrategy: Strategy<'t>
        + Strategy<'t, SNode = Self::PNode>
        + Strategy<'t, SPolicy = Self::PPolicy>
        + Strategy<'t, SPlayer = Self::PPlayer>
        + Strategy<'t, SAction = Self::PAction>;
}

/// A Solver will take a Profile and a Tree and iteratively consume/replace a new Profile on each iteration.
pub(crate) trait Solver<'t> {
    // required
    fn step(&'t self) -> &'t Self::SStep;
    fn tree(&'t self) -> &'t Self::STree;
    fn update_step(&mut self);
    fn update_tree(&mut self);

    // provided
    fn solve(&mut self) {
        for _ in 0..10_000 {
            self.update_tree();
            self.update_step();
        }
    }
    // (info) -> profile.strategy.policy update
    fn update_vector(&'t self, info: &'t Self::SInfo) -> Vec<(Self::SAction, Probability)> {
        info.available()
            .iter()
            .map(|action| **action)
            .zip(self.policy_vector(info).into_iter())
            .collect::<Vec<(Self::SAction, Probability)>>()
    }
    fn policy_vector(&'t self, info: &'t Self::SInfo) -> Vec<Probability> {
        let regrets = self.regret_vector(info);
        let sum = regrets.iter().sum::<Utility>();
        regrets.iter().map(|regret| regret / sum).collect()
    }
    fn regret_vector(&'t self, info: &'t Self::SInfo) -> Vec<Utility> {
        info.available()
            .iter()
            .map(|action| self.next_regret(info, action))
            .map(|regret| regret.max(Utility::MIN_POSITIVE))
            .collect()
    }
    // (info, action) -> regret
    fn gain(&'t self, root: &'t Self::SNode, action: &'t Self::SAction) -> Utility {
        self.step().cfactual_value(root, action) - self.step().expected_value(root)
    }
    fn next_regret(&'t self, info: &'t Self::SInfo, action: &'t Self::SAction) -> Utility {
        self.prev_regret(info, action) + self.curr_regret(info, action) //? Linear CFR weighting
    }
    fn curr_regret(&'t self, info: &'t Self::SInfo, action: &'t Self::SAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }
    fn prev_regret(&'t self, info: &'t Self::SInfo, action: &'t Self::SAction) -> Utility;

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<'t, NAction = Self::SAction> + Node<'t, NPlayer = Self::SPlayer>;
    type SInfo: Info<'t>
        + Info<'t, INode = Self::SNode>
        + Info<'t, IAction = Self::SAction>
        + Info<'t, IPlayer = Self::SPlayer>;
    type STree: Tree<'t>
        + Tree<'t, TInfo = Self::SInfo>
        + Tree<'t, TNode = Self::SNode>
        + Tree<'t, TEdge = Self::SAction>
        + Tree<'t, TPlayer = Self::SPlayer>;
    type SStrategy: Strategy<'t>
        + Strategy<'t, SNode = Self::SNode>
        + Strategy<'t, SAction = Self::SAction>
        + Strategy<'t, SPlayer = Self::SPlayer>
        + Strategy<'t, SPolicy = Self::SPolicy>;
    type SStep: Profile<'t>
        + Profile<'t, PStrategy = Self::SStrategy>
        + Profile<'t, PInfo = Self::SInfo>
        + Profile<'t, PNode = Self::SNode>
        + Profile<'t, PAction = Self::SAction>
        + Profile<'t, PPolicy = Self::SPolicy>
        + Profile<'t, PPlayer = Self::SPlayer>;
}
