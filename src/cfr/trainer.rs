use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::Probability;
use petgraph::graph::NodeIndex;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub struct Trainer(Profile, Tree);

impl Trainer {
    /// i'm making this a static method but in theory we could
    pub fn empty() -> Self {
        Self(Profile::empty(), Tree::empty())
    }
    pub fn train(&mut self, epochs: usize) {
        while self.0.step() <= epochs {
            for ref infoset in self.blocks() {
                if self.0.walker() == infoset.node().player() {
                    self.0.update_regret(infoset);
                    self.0.update_policy(infoset);
                }
            }
        }
        println!("{}", self.0);
    }

    /// the only thing we really need the tree for is to yield infosets for us to sample.
    /// these blocks can be sampled using whatever sampling scheme we like, it's
    /// encapsulated by the Tree itself and how it chooses to unfold from its Nodes.
    fn blocks(&mut self) -> Vec<Info> {
        self.1 = Tree::empty();
        self.dfs();
        self.1.infosets()
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    /// this is just a base case to handle the root node, presumably a Fn () -> Data.
    /// real-time search implementations may have root nodes provided by the caller.
    fn dfs(&mut self) {
        let root = Data::root();
        let head = self.attach(root);
        let head = self.1.graph_mut().add_node(head);
        for (tail, from) in self.sample(head) {
            self.unfold(tail, from, head);
        }
        assert!(head.index() == 0);
    }

    /// recursively build the Tree from the given Node, according to the distribution defined by Profile.
    /// we assert the Tree property of every non-root Node having exactly one parent Edge
    /// we construct the appropriate references in self.attach() to ensure safety.
    fn unfold(&mut self, head: Data, edge: Edge, root: NodeIndex) {
        let head = self.attach(head);
        let head = self.1.graph_mut().add_node(head);
        let edge = self.1.graph_mut().add_edge(root, head, edge);
        for (tail, from) in self.sample(head) {
            self.unfold(tail, from, head);
        }
        assert!(head.index() == edge.index() + 1);
    }

    /// attach a Node to the Tree,
    /// update the Profile to witness the new Node
    /// update the Tree to witness the new Node.
    fn attach(&mut self, data: Data) -> Node {
        let player = data.player().clone();
        let graph = self.1.graph_raw();
        let count = self.1.graph_ref().node_count();
        let index = NodeIndex::new(count);
        let node = Node::from((index, graph, data));
        if player != Player::Chance {
            self.0.witness(&node);
            self.1.witness(&node);
        }
        node
    }

    /// sample children of a Node, according to the distribution defined by Profile.
    /// we use external chance sampling, AKA explore all children of the traversing Player,
    /// while only probing a single child for non-traverser Nodes.
    /// this lands us in a high-variance, cheap-traversal, low-memory solution,
    /// compared to chance sampling, internal sampling, or full tree sampling.
    ///
    /// i think this could also be modified into a recursive CFR calcuation
    fn sample(&self, head: NodeIndex) -> Vec<(Data, Edge)> {
        let ref node = self.1.node(head);
        let mut sample = self.children(head);
        // terminal nodes have no children and we sample all possible actions for the traverser
        if node.player() == self.0.walker() || sample.is_empty() {
            sample
        }
        // choose random child uniformly. this is specific to the game of poker,
        // where each action at chance node/info/buckets is uniformly likely.
        else if node.player() == &Player::Chance {
            let ref mut rng = self.rng(node);
            let n = sample.len();
            let choice = rng.gen_range(0..n);
            let chosen = sample.remove(choice);
            vec![chosen];
            unreachable!("RPS specific")
        }
        // choose child according to reach probabilities in strategy profile.
        // on first iteration, this is equivalent to sampling uniformly.
        else {
            let ref mut rng = self.rng(node);
            let policy = sample
                .iter()
                .map(|(_, edge)| self.0.policy(node, edge))
                .collect::<Vec<Probability>>();
            let choice = WeightedIndex::new(policy)
                .expect("at least one policy > 0")
                .sample(rng);
            let chosen = sample.remove(choice);
            vec![chosen]
        }
    }
    /// produce the children of a Node.
    /// we may need some Trainer-level references to produce children
    /// so this is a method on Trainer for now.
    fn children(&self, head: NodeIndex) -> Vec<(Data, Edge)> {
        self.1.node(head).datum().spawn()
    }
    /// generate seed for PRNG. using hashing yields for deterministic, reproducable sampling
    /// for our Monte Carlo sampling. this may be better off as a function of
    /// (&Profile, &Node)      or
    /// (&Profile, &Bucket)
    /// but i like that it's here, since it's directly tied to tree-sampling. which is higher-level
    /// than either Tree or Profile.
    fn rng(&self, node: &Node) -> SmallRng {
        let ref mut hasher = DefaultHasher::new();
        node.bucket().hash(hasher);
        self.0.epochs().hash(hasher);
        SmallRng::seed_from_u64(hasher.finish())
    }
}
