use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::Probability;
use petgraph::graph::NodeIndex;

pub struct Trainer(Profile, Tree);

impl Trainer {
    /// i'm making this a static method but in theory we could
    pub fn empty() -> Self {
        Self(Profile::empty(), Tree::empty())
    }
    pub fn train(&mut self, epochs: usize) {
        while self.0.step() < epochs {
            for ref infoset in self.sample() {
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
    fn sample(&mut self) -> Vec<Info> {
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
        for (tail, from) in self.children(head) {
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
        for (tail, from) in self.children(head) {
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
    fn children(&self, head: NodeIndex) -> Vec<(Data, Edge)> {
        let node = self.1.node(head);
        let mut children = node.datum().spawn();
        if children.is_empty() {
            children
        } else if self.0.walker() == node.player() {
            children
        } else {
            use rand::distributions::Distribution;
            use rand::distributions::WeightedIndex;
            use rand::rngs::StdRng;
            use rand::SeedableRng;
            use std::collections::hash_map::DefaultHasher;
            use std::hash::Hash;
            use std::hash::Hasher;
            let ref mut hasher = DefaultHasher::new();
            self.0.epochs().hash(hasher);
            node.bucket().hash(hasher);
            let ref mut rng = StdRng::seed_from_u64(hasher.finish());
            let chance = children
                .iter()
                .map(|(_, edge)| self.0.policy(node, edge))
                .collect::<Vec<Probability>>();
            let choice = WeightedIndex::new(chance)
                .expect("at least one > 0")
                .sample(rng);
            let chosen = children.remove(choice);
            vec![chosen]
        }
    }
}
