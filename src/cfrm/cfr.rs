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
pub(crate) trait Node {
    // required
    fn parent(&self) -> Option<&Self>;
    fn precedent(&self) -> Option<&Self::NAction>;
    fn children(&self) -> &Vec<&Self>;
    fn available(&self) -> &Vec<&Self::NAction>;
    fn chooser(&self) -> &Self::NPlayer;
    fn utility(&self, player: &Self::NPlayer) -> Utility;

    // provided
    fn follow(&self, action: &Self::NAction) -> &Self {
        self.children()
            .iter()
            .find(|child| action == child.precedent().unwrap())
            .unwrap()
    }
    fn descendants(&self) -> Vec<&Self> {
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
pub(crate) trait Info {
    // required
    fn roots(&self) -> &Vec<&Self::INode>;

    // provided
    fn endpoints(&self) -> Vec<&Self::INode> {
        self.roots()
            .iter()
            .map(|node| node.descendants())
            .flatten()
            .collect()
    }
    fn available(&self) -> &Vec<&Self::IAction> {
        self.roots().iter().next().unwrap().available()
    }

    type IPlayer: Player;
    type IAction: Action<APlayer = Self::IPlayer>;
    type INode: Node<NAction = Self::IAction> + Node<NPlayer = Self::IPlayer>;
}

/// The owner all the Nodes, Actions, and Players in the context of a Solution. It also constrains the lifetime of references returned by its owned types. A vanilla implementation should build the full tree for small games. Monte Carlo implementations may sample paths conditional on given Profile, Solver, or other constraints. The only contract is that the Tree must be able to partition decision nodes into Info sets.
pub(crate) trait Tree {
    // required
    fn infos(&self) -> &Vec<Self::TInfo>;

    type TPlayer: Player;
    type TEdge: Action<APlayer = Self::TPlayer>;
    type TNode: Node<NAction = Self::TEdge> + Node<NPlayer = Self::TPlayer>;
    type TInfo: Info
        + Info<INode = Self::TNode>
        + Info<IAction = Self::TEdge>
        + Info<IPlayer = Self::TPlayer>;
}

/// A policy (P: node -> prob) is a distribution over A(Ii). Easily implemented as a HashMap<Aaction, Probability>.
pub(crate) trait Policy {
    // required
    fn weights(&self, action: &Self::PAction) -> Probability;

    type PAction: Action;
}

/// A strategy (σ: player -> policy) is a function that assigns a policy to each h ∈ H, and therefore Ii ∈ Ii. Easily implemented as a HashMap<Info, Policy>.
pub(crate) trait Strategy {
    // required
    fn policy(&self, node: &Self::SNode) -> &Self::SPolicy;

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<NAction = Self::SAction> + Node<NPlayer = Self::SPlayer>;
}

/// A profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
pub(crate) trait Profile {
    // required
    fn strategy(&self, player: &Self::PPlayer) -> &Self::PStrategy;

    // provided
    // utility calculations
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
        leaf.utility(root.chooser())
            * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
    }
    // probability calculations
    fn weight(&self, node: &Self::PNode, action: &Self::PAction) -> Probability {
        self.strategy(node.chooser()).policy(node).weights(action)
    }
    fn cfactual_reach(&self, node: &Self::PNode) -> Probability {
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
    type PInfo: Info
        + Info<INode = Self::PNode>
        + Info<IAction = Self::PAction>
        + Info<IPlayer = Self::PPlayer>;
    type PStrategy: Strategy
        + Strategy<SNode = Self::PNode>
        + Strategy<SPolicy = Self::PPolicy>
        + Strategy<SPlayer = Self::PPlayer>
        + Strategy<SAction = Self::PAction>;
}

/// A Solver will take a Profile and a Tree and iteratively consume/replace a new Profile on each iteration.
pub(crate) trait Solver {
    // required
    fn step(&self) -> &Self::SStep;
    fn tree(&self) -> &Self::STree;
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
    fn update_vector(&self, info: &Self::SInfo) -> Vec<(Self::SAction, Probability)> {
        info.available()
            .iter()
            .map(|action| **action)
            .zip(self.policy_vector(info).into_iter())
            .collect::<Vec<(Self::SAction, Probability)>>()
    }
    fn policy_vector(&self, info: &Self::SInfo) -> Vec<Probability> {
        let regrets = self.regret_vector(info);
        let sum = regrets.iter().sum::<Utility>();
        regrets.iter().map(|regret| regret / sum).collect()
    }
    fn regret_vector(&self, info: &Self::SInfo) -> Vec<Utility> {
        info.available()
            .iter()
            .map(|action| self.next_regret(info, action))
            .map(|regret| regret.max(Utility::MIN_POSITIVE))
            .collect()
    }
    // (info, action) -> regret
    fn gain(&self, root: &Self::SNode, action: &Self::SAction) -> Utility {
        self.step().cfactual_value(root, action) - self.step().expected_value(root)
    }
    fn next_regret(&self, info: &Self::SInfo, action: &Self::SAction) -> Utility {
        self.prev_regret(info, action) + self.curr_regret(info, action) //? Linear CFR weighting
    }
    fn curr_regret(&self, info: &Self::SInfo, action: &Self::SAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }
    fn prev_regret(&self, info: &Self::SInfo, action: &Self::SAction) -> Utility;

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<NAction = Self::SAction> + Node<NPlayer = Self::SPlayer>;
    type SInfo: Info
        + Info<INode = Self::SNode>
        + Info<IAction = Self::SAction>
        + Info<IPlayer = Self::SPlayer>;
    type STree: Tree
        + Tree<TInfo = Self::SInfo>
        + Tree<TNode = Self::SNode>
        + Tree<TEdge = Self::SAction>
        + Tree<TPlayer = Self::SPlayer>;
    type SStrategy: Strategy
        + Strategy<SNode = Self::SNode>
        + Strategy<SAction = Self::SAction>
        + Strategy<SPlayer = Self::SPlayer>
        + Strategy<SPolicy = Self::SPolicy>;
    type SStep: Profile
        + Profile<PStrategy = Self::SStrategy>
        + Profile<PInfo = Self::SInfo>
        + Profile<PNode = Self::SNode>
        + Profile<PAction = Self::SAction>
        + Profile<PPolicy = Self::SPolicy>
        + Profile<PPlayer = Self::SPlayer>;
}
