use structs::Trainer;

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

#[tokio::main]
async fn main() {
    let mut trainer = Trainer::new();
    traits::Trainer::train(&mut trainer, 50);
}

pub mod traits {
    use petgraph::graph::DiGraph;
    use petgraph::graph::NodeIndex;
    use petgraph::Direction::Incoming;
    use petgraph::Direction::Outgoing;
    use std::hash::Hash;

    type Utility = f32;
    type Probability = f32;

    pub(crate) trait Action<'t>: 't + Sized + Eq + Hash + Copy {}
    pub(crate) trait Bucket<'t>: 't + Sized + Eq + Hash {}
    pub(crate) trait Player<'t>: 't + Sized + Eq {}

    /// the inner state of a node, abstracted over the type of action and bucket
    pub(crate) trait Local {}

    /// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
    pub(crate) trait Node<'t, L, A, B, C>
    where
        Self: 't + Sized,
        A: Action<'t>,
        B: Bucket<'t>,
        C: Player<'t>,
        L: Local,
    {
        // required
        fn payoff(&'t self, player: &'t C) -> Utility;
        fn player(&'t self) -> &'t C;
        fn bucket(&'t self) -> &'t B;
        fn local(&'t self) -> &'t L;
        fn index(&'t self) -> &'t NodeIndex;
        fn graph(&'t self) -> &'t DiGraph<Self, A>;

        // walkability
        fn parent(&'t self) -> Option<&'t Self> {
            self.graph()
                .neighbors_directed(*self.index(), Incoming)
                .next()
                .map(|index| {
                    self.graph()
                        .node_weight(index)
                        .expect("tree property: if incoming edge, then parent")
                })
        }
        fn children(&'t self) -> Vec<&'t Self> {
            self.graph()
                .neighbors_directed(*self.index(), Outgoing)
                .map(|c| {
                    self.graph()
                        .node_weight(c)
                        .expect("tree property: if outgoing edge, then child")
                })
                .collect()
        }
        fn incoming(&'t self) -> Option<&'t A> {
            self.graph()
                .edges_directed(*self.index(), Incoming)
                .next()
                .map(|e| e.weight())
        }
        fn outgoing(&'t self) -> Vec<&'t A> {
            self.graph()
                .edges_directed(*self.index(), Outgoing)
                .map(|e| e.weight())
                .collect()
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
        fn follow(&'t self, edge: &'t A) -> &'t Self {
            self.children()
                .iter()
                .find(|child| edge == child.incoming().unwrap())
                .unwrap()
        }
    }

    /// distribution over indistinguishable nodes, abstracted over the type of node
    pub(crate) trait Info<'t, N, L, A, B, C>
    where
        N: Node<'t, L, A, B, C>,
        A: Action<'t>,
        B: Bucket<'t>,
        C: Player<'t>,
        L: Local,
    {
        // required
        fn roots(&'t self) -> Vec<&'t N>;

        fn player(&'t self) -> &'t C {
            self.roots().iter().next().unwrap().player()
        }
        fn bucket(&'t self) -> &'t B {
            self.roots().iter().next().unwrap().bucket()
        }
        fn outgoing(&'t self) -> Vec<&'t A> {
            self.roots().iter().next().unwrap().outgoing()
        }
    }

    /// a tree will own the graph and infosets
    pub(crate) trait Tree<'t, I, N, L, A, B, C>
    where
        I: Info<'t, N, L, A, B, C>,
        N: Node<'t, L, A, B, C>,
        A: Action<'t>,
        B: Bucket<'t>,
        C: Player<'t>,
        L: Local,
    {
        // required
        fn infosets(&'t self) -> Vec<&'t I>;
    }

    /// a policy π is a distribution over actions given a bucket. Equivalently a vector indexed by action ∈ A
    pub(crate) trait Distribution<'t, A>
    where
        A: Action<'t>,
    {
        // required
        fn weight(&self, action: &A) -> Probability;
        fn sample(&self) -> &A;
    }

    /// a strategy σ is a policy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
    pub(crate) trait Strategy<'t, D, A, B>
    where
        D: Distribution<'t, A>,
        A: Action<'t>,
        B: Bucket<'t>,
    {
        // required
        fn policy(&self, bucket: &B) -> &D;
    }

    /// a profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
    pub(crate) trait Profile<'t, S, D, N, L, A, B, C>
    where
        S: Strategy<'t, D, A, B>,
        D: Distribution<'t, A>,
        N: Node<'t, L, A, B, C>,
        A: Action<'t>,
        B: Bucket<'t>,
        C: Player<'t>,
        L: Local,
    {
        // required
        fn strategy(&self, player: &C) -> &S;

        // provided
        fn gain(&self, root: &'t N, action: &'t A) -> Utility {
            // self.cfactual_value(root, action) - self.expected_value(root)
            let cfactual = self.cfactual_value(root, action);
            let expected = self.expected_value(root);
            cfactual - expected
        }
        fn cfactual_value(&self, root: &'t N, action: &'t A) -> Utility {
            self.cfactual_reach(root)
                * root //                                       suppose you're here on purpose, counterfactually
                    .follow(action) //                          suppose you're here on purpose, counterfactually
                    .descendants() //                           O(depth) recursive downtree
                    .iter() //                                  duplicated calculation
                    .map(|leaf| self.relative_value(root, leaf))
                    .sum::<Utility>()
        }
        fn expected_value(&self, root: &'t N) -> Utility {
            self.strategy_reach(root)
                * root
                    .descendants() //                           O(depth) recursive downtree
                    .iter() //                                  duplicated calculation
                    .map(|leaf| self.relative_value(root, leaf))
                    .sum::<Utility>()
        }
        fn relative_value(&self, root: &'t N, leaf: &'t N) -> Utility {
            leaf.payoff(root.player())
                * self.relative_reach(root, leaf)
                * self.sampling_reach(root, leaf)
        }
        // probability calculations
        fn weight(&self, node: &'t N, action: &A) -> Probability {
            self.strategy(node.player())
                .policy(node.bucket())
                .weight(action)
        }
        fn cfactual_reach(&self, node: &'t N) -> Probability {
            match node.parent() {
                None => 1.0,
                Some(parent) => {
                    self.cfactual_reach(parent)
                        * if node.player() == parent.player() {
                            1.0
                        } else {
                            self.weight(
                                parent,
                                node.incoming().expect("if has parent, then has incoming"),
                            )
                        }
                }
            }
        }
        fn strategy_reach(&self, node: &'t N) -> Probability {
            match node.parent() {
                None => 1.0,
                Some(parent) => {
                    let edge = node.incoming().expect("if has parent, then has incoming");
                    self.strategy_reach(parent) * self.weight(parent, edge)
                }
            }
        }
        fn relative_reach(&self, root: &'t N, leaf: &'t N) -> Probability {
            if root.bucket() == leaf.bucket() {
                1.0
            } else {
                let node = leaf.parent().expect("if has parent, then has incoming");
                let edge = leaf.incoming().expect("if has parent, then has incoming");
                self.relative_reach(root, node) * self.weight(node, edge)
            }
        }
        fn sampling_reach(&self, _: &'t N, _: &'t N) -> Probability {
            1.0
        }
    }

    /// an optimizer updates profile to minimize regret, and updates regrets from existing profiles.
    pub(crate) trait Optimizer<'t, P, S, D, I, N, L, A, B, C>
    where
        P: Profile<'t, S, D, N, L, A, B, C>,
        S: Strategy<'t, D, A, B>,
        D: Distribution<'t, A>,
        I: Info<'t, N, L, A, B, C>,
        N: Node<'t, L, A, B, C>,
        A: Action<'t>,
        B: Bucket<'t>,
        C: Player<'t>,
        L: Local,
    {
        // required
        fn profile(&self) -> &P;
        fn update_regret(&mut self, info: &I);
        fn update_policy(&mut self, info: &I);
        // regret storge and calculation
        fn running_regret(&self, info: &'t I, action: &'t A) -> Utility;
        fn instant_regret(&self, info: &'t I, action: &'t A) -> Utility {
            info.roots()
                .iter()
                .map(|root| self.profile().gain(root, action))
                .sum::<Utility>()
        }
        fn matching_regret(&self, info: &'t I, action: &'t A) -> Utility {
            let running = self.running_regret(info, action);
            let instant = self.instant_regret(info, action);
            (running + instant).max(Utility::MIN_POSITIVE)
        }
        // policy calculation via regret matching +
        fn policy_vector(&self, info: &'t I) -> Vec<(A, Probability)> {
            let regrets = info
                .outgoing()
                .iter()
                .map(|action| (**action, self.running_regret(info, action)))
                .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
                .collect::<Vec<(A, Probability)>>();
            let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
            let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
            policy
        }
        fn regret_vector(&self, info: &'t I) -> Vec<(A, Utility)> {
            info.outgoing()
                .iter()
                .map(|action| (**action, self.matching_regret(info, action)))
                .collect()
        }
    }

    /// trainer will update regrets and profile in a mutable loop
    pub(crate) trait Trainer<'t, O, P, S, D, T, I, N, L, A, B, C>
    where
        O: Optimizer<'t, P, S, D, I, N, L, A, B, C>,
        P: Profile<'t, S, D, N, L, A, B, C>,
        S: Strategy<'t, D, A, B>,
        D: Distribution<'t, A>,
        T: Tree<'t, I, N, L, A, B, C>,
        I: Info<'t, N, L, A, B, C>,
        N: Node<'t, L, A, B, C>,
        A: Action<'t>,
        B: Bucket<'t>,
        C: Player<'t>,
        L: Local,
    {
        // required
        fn train(&mut self, n: usize);
    }
}

pub mod structs {
    use super::traits;
    use petgraph::graph::DiGraph;
    use petgraph::graph::EdgeIndex;
    use petgraph::graph::NodeIndex;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::ptr::NonNull;

    type Utility = f32;
    type Probability = f32;

    /// buckets
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
    pub(crate) enum Bucket {
        P1,
        P2,
        Ignore,
    }
    impl traits::Bucket<'_> for Bucket {}

    /// players
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
    pub(crate) enum Player {
        P1,
        P2,
        Chance,
    }
    impl traits::Player<'_> for Player {}

    /// actions
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
    pub(crate) enum Action {
        RK,
        PA,
        SC,
    }
    impl traits::Action<'_> for Action {}

    /// treat this as the topologically sorted number of RPS states
    pub(crate) struct Local(usize);
    impl Local {
        /// global tree geenration
        pub fn root() -> Self {
            Self(0)
        }
        /// abstraction
        pub fn bucket(&self) -> &Bucket {
            match self.0 {
                00 => &Bucket::P1,
                01..=03 => &Bucket::P2,
                04..=12 => &Bucket::Ignore,
                _ => unreachable!(),
            }
        }
        /// attribution
        pub fn player(&self) -> &Player {
            match self.0 {
                00 => &Player::P1,
                01..=03 => &Player::P2,
                04..=12 => &Player::Chance,
                _ => unreachable!(),
            }
        }
        /// local tree generation
        pub fn spawn(&self) -> Vec<(Self, Action)> {
            match self.0 {
                // P1 moves
                00 => vec![
                    (Self(01), Action::RK),
                    (Self(02), Action::PA),
                    (Self(03), Action::SC),
                ],
                // P2 moves
                01 => vec![
                    (Self(04), Action::RK),
                    (Self(05), Action::PA),
                    (Self(06), Action::SC),
                ],
                02 => vec![
                    (Self(07), Action::RK),
                    (Self(08), Action::PA),
                    (Self(09), Action::SC),
                ],
                03 => vec![
                    (Self(10), Action::RK),
                    (Self(11), Action::PA),
                    (Self(12), Action::SC),
                ],
                // terminal nodes
                04..=12 => Vec::new(),
                //
                _ => unreachable!(),
            }
        }
        pub fn payoff(&self, player: &Player) -> Utility {
            const R_WIN: Utility = 1.;
            const P_WIN: Utility = 1.;
            const S_WIN: Utility = 5.; // we can modify payoffs to verify convergence
            let direction = match player {
                Player::P1 => 0. + 1.,
                Player::P2 => 0. - 1.,
                _ => unreachable!(),
            };
            let payoff = match self.0 {
                07 => 0. + P_WIN, // P > R
                05 => 0. - P_WIN, // R < P
                06 => 0. + S_WIN, // R > S
                11 => 0. + S_WIN, // S > P
                10 => 0. - S_WIN, // S < R
                09 => 0. - S_WIN, // P < S
                04 | 08 | 12 => 0.0,
                00..=03 => unreachable!("eval at terminal node, depth > 1"),
                _ => unreachable!(),
            };
            direction * payoff
        }
    }
    impl traits::Local for Local {}

    /// nodes
    pub(crate) struct Node {
        local: Local,
        index: NodeIndex,
        graph: NonNull<DiGraph<Self, Action>>,
    }
    impl traits::Node<'_, Local, Action, Bucket, Player> for Node {
        fn local(&self) -> &Local {
            &self.local
        }
        fn index(&self) -> &NodeIndex {
            &self.index
        }
        fn graph(&self) -> &DiGraph<Self, Action> {
            unsafe { self.graph.as_ref() }
        }
        fn bucket(&self) -> &Bucket {
            self.local().bucket()
        }
        fn player(&self) -> &Player {
            self.local().player()
        }
        fn payoff(&self, player: &Player) -> Utility {
            self.local().payoff(player)
        }
    }

    /// info sets
    pub(crate) struct Info {
        roots: Vec<NodeIndex>,
        graph: NonNull<DiGraph<Node, Action>>,
    }
    impl Info {
        pub fn add(&mut self, node: &Node) {
            self.roots.push(*traits::Node::index(node));
        }
        fn graph(&self) -> &DiGraph<Node, Action> {
            unsafe { self.graph.as_ref() }
        }
    }
    impl traits::Info<'_, Node, Local, Action, Bucket, Player> for Info {
        fn roots(&self) -> Vec<&Node> {
            self.roots
                .iter()
                .map(|i| self.graph().node_weight(*i).unwrap())
                .collect()
        }
    }

    /// trees
    pub(crate) struct Tree {
        index: NodeIndex,
        graph: Box<DiGraph<Node, Action>>,
        infos: HashMap<Bucket, Info>,
    }
    impl Tree {
        pub fn new() -> Self {
            let mut this = Self {
                index: NodeIndex::new(0),
                graph: Box::new(DiGraph::new()),
                infos: HashMap::new(),
            };
            this.insert(Local::root());
            this.explore();
            this.bucketize();
            this
        }
        fn explore(&mut self) {
            while self.index.index() < self.graph.node_count() {
                for (child, edge) in self.spawn() {
                    self.attach(child, edge);
                }
                self.advance();
            }
        }
        fn advance(&mut self) {
            self.index = NodeIndex::new(self.index.index() + 1);
        }
        fn bucketize(&mut self) {
            for node in self
                .graph
                .node_weights()
                .filter(|n| traits::Node::player(*n) != &Player::Chance)
            {
                self.infos
                    .entry(*traits::Node::bucket(node))
                    .or_insert_with(|| Info {
                        roots: Vec::new(),
                        graph: NonNull::from(&*self.graph),
                    })
                    .add(node);
            }
        }
        fn insert(&mut self, local: Local) -> NodeIndex {
            let i_node = self.graph.add_node(Node {
                local,
                index: NodeIndex::new(self.graph.node_count()),
                graph: NonNull::from(&*self.graph),
            });
            i_node
        }
        fn attach(&mut self, local: Local, edge: Action) -> EdgeIndex {
            let i_node = self.insert(local);
            let i_edge = self.graph.add_edge(self.index, i_node, edge);
            i_edge
        }
        fn spawn(&self) -> Vec<(Local, Action)> {
            traits::Node::local(
                self.graph
                    .node_weight(self.index)
                    .expect("self.point will be behind self.graph.node_count"),
            )
            .spawn()
        }
    }
    impl traits::Tree<'_, Info, Node, Local, Action, Bucket, Player> for Tree {
        fn infosets(&self) -> Vec<&Info> {
            self.infos.values().collect()
        }
    }

    // distributions, strategies and profiles can all be implemented with a HashMap

    /// distributions
    type Distribution = HashMap<Action, Probability>;
    impl traits::Distribution<'_, Action> for Distribution {
        fn weight(&self, action: &Action) -> Probability {
            *self
                .get(action)
                .expect("policy initialized across (bucket, action) set")
        }
        fn sample(&self) -> &Action {
            self.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0
        }
    }

    /// strategy
    type Strategy = HashMap<Bucket, Distribution>;
    impl traits::Strategy<'_, Distribution, Action, Bucket> for Strategy {
        fn policy(&self, bucket: &Bucket) -> &Distribution {
            self.get(bucket)
                .expect("policy initialized across bucket set")
        }
    }

    /// profile
    type Profile = HashMap<Bucket, Distribution>;
    impl traits::Profile<'_, Self, Distribution, Node, Local, Action, Bucket, Player> for Profile {
        fn strategy(&self, _: &Player) -> &Strategy {
            &self
        }
    }

    /// optimizer'
    pub(crate) struct Optimizer {
        time: usize,
        average: HashMap<Bucket, HashMap<Action, Probability>>,
        profile: HashMap<Bucket, HashMap<Action, Probability>>,
        regrets: HashMap<Bucket, HashMap<Action, Utility>>,
    }
    impl Optimizer {
        pub fn new(tree: &Tree) -> Self {
            let mut average = HashMap::new();
            let mut profile = HashMap::new();
            let mut regrets = HashMap::new();
            for info in traits::Tree::infosets(tree) {
                let n = traits::Info::outgoing(info).len();
                let weight = 1.0 / n as Probability;
                let regret = 0.0;
                let bucket = traits::Info::bucket(info);
                for action in traits::Info::outgoing(info) {
                    average
                        .entry(*bucket)
                        .or_insert_with(HashMap::new)
                        .insert(*action, weight);
                    profile
                        .entry(*bucket)
                        .or_insert_with(HashMap::new)
                        .insert(*action, weight);
                    regrets
                        .entry(*bucket)
                        .or_insert_with(HashMap::new)
                        .insert(*action, regret);
                }
            }
            Self {
                time: 0,
                average,
                profile,
                regrets,
            }
        }
    }
    impl
        traits::Optimizer<
            '_,
            Profile,
            Strategy,
            Distribution,
            Info,
            Node,
            Local,
            Action,
            Bucket,
            Player,
        > for Optimizer
    {
        fn profile(&self) -> &Strategy {
            &self.profile
        }
        fn update_regret(&mut self, info: &Info) {
            for (ref action, regret) in self.regret_vector(info) {
                let running = self
                    .regrets
                    .get_mut(traits::Info::bucket(info))
                    .expect("regret initialized for infoset")
                    .get_mut(action)
                    .expect("regret initialized for actions");
                *running = regret;
            }
        }
        fn update_policy(&mut self, info: &Info) {
            for (ref action, weight) in self.policy_vector(info) {
                let current = self
                    .profile
                    .get_mut(traits::Info::bucket(info))
                    .expect("policy initialized for infoset")
                    .get_mut(action)
                    .expect("policy initialized for actions");
                let average = self
                    .average
                    .get_mut(traits::Info::bucket(info))
                    .expect("average initialized for infoset")
                    .get_mut(action)
                    .expect("average initialized for infoset");
                *current = weight;
                *average *= self.time as Probability;
                *average += weight;
                *average /= self.time as Probability + 1.;
            }
            self.time += 1;
        }
        fn running_regret(&self, info: &Info, action: &Action) -> Utility {
            *self
                .regrets
                .get(traits::Info::bucket(info))
                .expect("regret initialized for infoset")
                .get(action)
                .expect("regret initialized for actions")
        }
    }

    /// trainer
    pub(crate) struct Trainer {
        tree: Tree,
        optimizer: Optimizer,
    }
    impl Trainer {
        pub fn new() -> Self {
            let tree = Tree::new();
            let optimizer = Optimizer::new(&tree);
            Self { optimizer, tree }
        }
    }
    impl
        traits::Trainer<
            '_,
            Optimizer,
            Profile,
            Strategy,
            Distribution,
            Tree,
            Info,
            Node,
            Local,
            Action,
            Bucket,
            Player,
        > for Trainer
    {
        fn train(&mut self, n: usize) {
            for t in 0..n {
                for info in traits::Tree::infosets(&self.tree)
                    .iter()
                    .rev()
                    .filter(|i| traits::Info::player(**i) != &Player::Chance)
                {
                    traits::Optimizer::update_regret(&mut self.optimizer, info);
                    traits::Optimizer::update_policy(&mut self.optimizer, info);
                }
                if t % 1 == 0 {
                    for (bucket, distribution) in self.optimizer.average.iter() {
                        for (action, weight) in distribution.iter() {
                            println!("B{:?}  {:?} : {:.3?} @ t{:?}", bucket, action, weight, t);
                        }
                        println!();
                        break;
                    }
                }
            }
        }
    }
}
