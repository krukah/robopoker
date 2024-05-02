#![allow(dead_code)]

/// Regret Minimization in Games with Incomplete Information. Advances in Neural Information Processing Systems, 20.
/// Zinkevich, M., Bowling, M., Burch, N., Cao, Y., Johanson, M., Tamblyn, I., & Rocco, M. (2007).

// Marker types
type Utility = f32;
type Probability = f32;

// A finite set N of players, including chance
trait Player {}

// A finite set of possible actions
trait Action {
    type Player;

    fn player(&self) -> &Self::Player;
    fn belongs(&self, player: &Self::Player) -> bool;
}

// Omnipotent, complete state of current game
trait Node {
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    // fn parent(&self) -> Option<&Self>;
    fn value(&self, _: &Self::Player) -> Utility;
    fn player(&self) -> &Self::Player;
    fn history(&self) -> Vec<&Self::Action>;
    fn available(&self) -> Vec<&Self::Action>;
    fn children(&self) -> Vec<&Self>;

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
}

// All known information at a given node, up to any abstractions. Think of it as a distribution over the unknown game state.
trait Info {
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    fn possibilities(&self) -> Vec<&Self::Node>;

    fn endpoints(&self) -> Vec<&Self::Node> {
        self.possibilities()
            .iter()
            .map(|node| node.descendants())
            .flatten()
            .collect()
    }
    fn available(&self) -> Vec<&Self::Action> {
        self.possibilities().into_iter().next().unwrap().available()
    }
    fn player(&self) -> &Self::Player {
        self.possibilities().iter().next().unwrap().player()
    }
}

// A policy is a distribution over A(Ii)
trait Policy {
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    fn weight(&self, action: &Self::Action) -> Probability;
}

// A strategy of player i σi in an extensive game is a function that assigns a policy to each h ∈ H, therefore Ii ∈ Ii
trait Strategy {
    type Policy: Policy<Player = Self::Player, Action = Self::Action>;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    fn policy(&self, node: &Self::Node) -> &Self::Policy;
}

// A profile σ consists of a strategy for each player, σ1,σ2,..., equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
trait Profile {
    type Strategy: Strategy<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
    >;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    /// Return a Profile where info.player's strategy is to play P(action)= 100%
    fn always(&self, action: &Self::Action) -> Self;
    /// Return a Profile where info.player's strategy is given
    fn replace(&self, strategy: &Self::Strategy) -> Self;
    /// Return the strategy for player i
    fn strategy(&self, player: &Self::Player) -> &Self::Strategy;
    /// Return the set of strategies for P_i
    fn strategies(&self) -> Vec<&Self::Strategy>;

    /// EV for info.player iff players play according to &self
    fn expected_value(&self, info: &Self::Info /* player */) -> Utility {
        info.endpoints()
            .iter()
            .map(|leaf| leaf.value(info.player()) * self.reach(leaf))
            .sum()
    }
    /// EV for info.player iff players play according to &self BUT info.player plays according to P(action)= 100%.
    /// i think we can interpret this as a dot product/measure of alignment between
    /// optimal P_i strategy and current P_i strategy, given a fixed info set and fixed opponent strategy
    fn cfactual_value(&self, info: &Self::Info /* player */) -> Utility {
        info.possibilities()
            .iter()
            .map(|root| {
                root.descendants()
                    .iter()
                    .map(|leaf| {
                        leaf.value(info.player())       // V ( LEAF )
                            * self.exterior_reach(root) // P ( ROOT | player tried to reach INFO )
                            * self.relative_reach(root, leaf) // P ( ROOT -> LEAF )
                    })
                    .sum::<Utility>()
            })
            .sum::<Utility>()
            / info
                .possibilities()
                .iter()
                .map(|root| self.reach(root))
                .sum::<Utility>() //? DIV BY ZERO
    }
    // reach probabilities
    fn reach(&self, node: &Self::Node) -> Probability {
        node.history()
            .iter()
            .map(|action| self.strategy(action.player()).policy(node).weight(action))
            .product()
    }
    fn exterior_reach(&self, node: &Self::Node) -> Probability {
        node.history()
            .iter()
            .filter(|action| !!!action.belongs(node.player()))
            .map(|action| self.strategy(action.player()).policy(node).weight(action))
            .product()
    }
    fn relative_reach(&self, root: &Self::Node, leaf: &Self::Node) -> Probability {
        self.reach(leaf) / self.reach(root) //? DIV BY ZERO
    }
}

// Training happens over discrete time steps, so we'll index steps into it's own data structure.xz
trait Step {
    type Profile: Profile<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
        Strategy = Self::Strategy,
    >;
    type Strategy: Strategy<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
    >;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    fn new(profile: Self::Profile) -> Self;
    fn profile(&self) -> &Self::Profile; //? mutable or immutable?

    /// aka instantaneous regret.
    fn gain(&self, info: &Self::Info, action: &Self::Action) -> Utility {
        self.profile().always(action).cfactual_value(info) - self.profile().cfactual_value(info)
    }
}

// A full solver has a sequence of steps, and a final profile
trait Solver {
    type Step: Step<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
        Strategy = Self::Strategy,
        Profile = Self::Profile,
    >;
    type Profile: Profile<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
        Strategy = Self::Strategy,
    >;
    type Strategy: Strategy<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
    >;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;

    // fn info(&self) -> &Self::Info;
    fn steps(&self) -> &mut Vec<Self::Step>;
    fn next_profile(&self) -> Self::Profile;

    /// aka average cumulative regret.
    fn regret(&self, info: &Self::Info, action: &Self::Action) -> Utility {
        self.steps()
            .iter()
            .map(|step| step.gain(info, action))
            .sum::<Utility>()
            / self.num_steps() as Utility //? DIV BY ZERO
    }
    /// Loops over simple n_iter < max_iter convergence criteria and returns ~ Nash Equilibrium
    fn solve(&self) -> &Self::Profile {
        while let Some(step) = self.next() {
            self.steps().push(step);
        }
        self.steps().last().unwrap().profile()
    }
    /// Generate the next Step of the solution as a pure function of current state
    fn next(&self) -> Option<Self::Step> {
        if self.num_steps() < self.max_steps() {
            Some(Self::Step::new(self.next_profile()))
        } else {
            None
        }
    }
    /// Convergence progress
    fn num_steps(&self) -> usize {
        self.steps().len()
    }
    fn max_steps(&self) -> usize {
        10_000
    }
}
