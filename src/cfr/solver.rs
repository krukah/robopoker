use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::cfr::profile::Profile;
use crate::cfr::tree::Tree;
use crate::Probability;
use crate::Utility;
use rand::rngs::SmallRng;
use rand::SeedableRng;

pub struct Solution {
    t: usize,
    regrets: Profile,
    current: Profile,
    average: Profile,
}

/// CFR Solver
impl Solution {
    pub fn new() -> Self {
        Self {
            t: 0,
            average: Profile::new(),
            current: Profile::new(),
            regrets: Profile::new(),
        }
    }
    pub fn report(&self) {
        const CHECKPOINT: usize = 1_000;
        if self.t % CHECKPOINT == 0 {
            println!("T{}", self.t);
            for (bucket, strategy) in self.mean().strategies() {
                for (action, weight) in strategy.0.iter() {
                    println!("Bucket {:?}  {:?}: {:.4?}", bucket, action, weight);
                }
                break;
            }
        }
    }
    pub fn mean(&self) -> &Profile {
        &self.average
    }
    pub fn solve(&mut self, epochs: usize) {
        while self.t < epochs {
            for block in self.sample().blocks() {
                if self.walker() == block.node().player() {
                    self.update_regret(block);
                    self.update_policy(block);
                } else {
                    continue;
                }
            }
            self.report();
            self.t += 1;
        }
    }

    /// generate a new MC tree using the current profile as a sampling distribution
    /// - use DFS to expand
    /// - use the current profile as a sampling distribution
    /// - initialize unreached Buckets with normalized probabilities
    fn sample(&self) -> Tree {
        todo!("sample new MC tree. use DFS to expand, self.profile to sample");
        todo!("maybe initialize unreached Buckets with normalized probabilities");
        /*
        pub fn new() -> Self {
            let mut this = Self {
                infos: HashMap::new(),
                graph: Box::new(DiGraph::new()),
            };
            this.dfs();
            this.bucketize();
            this
        }

        fn dfs(&mut self) {
            let root = (Self::root(), None, NodeIndex::from(0));
            let mut parents = vec![root];
            while let Some(parent) = parents.pop() {
                let mut children = self.spawn(&parent.0);
                let (data, from, head) = parent;
                let node = self.engulf(data); // , index
                let tail = self.attach(node, from, head); // , mut index
                while let Some(child) = children.pop() {
                    let data = child.data;
                    let edge = Some(child.edge);
                    parents.push((data, edge, tail));
                }
            }
        }

        fn bucketize(&mut self) {
            for node in self.graph.node_weights() {
                let index = node.index();
                let player = node.player();
                let bucket = node.bucket();
                if player == &Player::Chance {
                    continue;
                } else {
                    match self.infos.get_mut(bucket) {
                        Some(info) => info.push(index),
                        None => {
                            let info = Info::from((index, self.graph()));
                            let bucket = bucket.to_owned();
                            self.infos.insert(bucket, info);
                        }
                    }
                }
            }
        }
         */
    }

    /// update regrets via regret matching
    fn update_regret(&mut self, info: &Info) {
        for (ref action, ref mut regret) in self.regret_vector(info) {
            let bucket = info.node().bucket();
            let running = self.regrets.get_mut(bucket, action);
            std::mem::swap(running, regret);
        }
    }
    /// update policy via cumulative regrets
    fn update_policy(&mut self, info: &Info) {
        for (ref action, weight) in self.policy_vector(info) {
            let bucket = info.node().bucket();
            let current = self.current.get_mut(bucket, action);
            let average = self.average.get_mut(bucket, action);
            *current = weight;
            *average *= self.t as Probability;
            *average += weight;
            *average /= self.t as Probability + 1.;
        }
    }

    /// policy calculation via cumulative regrets
    fn policy_vector(&self, infonode: &Info) -> Vec<(Edge, Probability)> {
        let regrets = infonode
            .node()
            .outgoing()
            .iter()
            .map(|action| (**action, self.running_regret(infonode, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(Edge, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    /// regret storage and calculation
    fn regret_vector(&self, infonode: &Info) -> Vec<(Edge, Utility)> {
        infonode
            .node()
            .outgoing()
            .into_iter()
            .map(|action| (*action, self.matched_regret(infonode, action)))
            .collect()
    }

    /// regret storage and calculation
    fn matched_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(infonode, action);
        let instant = self.instant_regret(infonode, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        let bucket = infonode.node().bucket();
        let regret = self.regrets.get_ref(bucket, action);
        *regret
    }
    fn instant_regret(&self, infonode: &Info, action: &Edge) -> Utility {
        infonode
            .roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }

    // marginal counterfactual gain over strategy EV
    fn gain(&self, root: &Node, edge: &Edge) -> Utility {
        let cfactual = self.cfactual_value(root, edge);
        let expected = self.expected_value(root); // should hoist outside of action/edge loop
        cfactual - expected
    }
    fn cfactual_value(&self, root: &Node, edge: &Edge) -> Utility {
        self.current.cfactual_reach(root) * self.visiting_value(root.follow(edge))
    }
    fn expected_value(&self, root: &Node) -> Utility {
        self.current.expected_reach(root) * self.visiting_value(root)
    }
    fn visiting_value(&self, root: &Node) -> Utility {
        self.explore(root)
            .iter()
            .map(|leaf| self.relative_value(root, leaf))
            .sum::<Utility>()
    }
    fn relative_value(&self, root: &Node, leaf: &Node) -> Utility {
        Node::payoff(root, leaf)
            * self.current.relative_reach(root, leaf)
            * self.current.sampling_reach(root, leaf)
    }

    /// external sampling helper method derived from epoch
    /// basically just used to alternate between P1 and P2
    fn walker(&self) -> &Player {
        match self.t % 2 {
            0 => &Player::P1,
            _ => &Player::P2,
        }
    }
    /// Given existing Tree/Graph/Node, implement external sampling tree search.
    /// the walker/player comparision is the selection mechanism:
    /// - explores all children if the walker is this epoch's traverser
    /// - explores a single randomly selected child otherwise
    fn explore<'tree>(&self, node: &'tree Node) -> Vec<&'tree Node> {
        if 0 == node.children().len() {
            vec![node]
        } else if self.walker() == node.player() {
            self.explore_all(node)
        } else {
            self.explore_one(node)
        }
    }
    /// explores all children of the current node
    /// high branching factor -> exploring all our options
    fn explore_all<'tree>(&self, node: &'tree Node) -> Vec<&'tree Node> {
        node.children()
            .iter()
            .map(|child| self.explore(child))
            .flatten()
            .collect()
    }
    /// explores a single randomly selected child
    /// low branching factor -> prevent compinatoric explosion.
    ///
    /// implementation assumes we'll have a policy for this Node/Bucket/Info, i.e.
    /// - static Tree
    /// - dynamic terminal Node / descendant selection
    fn explore_one<'tree>(&self, node: &'tree Node) -> Vec<&'tree Node> {
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;
        let seed = [(self.t + node.index().index()) as u8; 32];
        let ref mut rng = SmallRng::from_seed(seed);
        let ref weights = node
            .outgoing()
            .iter()
            .map(|edge| self.current.weight(node, edge))
            .collect::<Vec<Probability>>();
        let child = WeightedIndex::new(weights)
            .expect("same length, at least one > 0")
            .sample(rng);
        let child = node.children().remove(child); // kidnapped!
        self.explore(child)
    }
}
