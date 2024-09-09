#![allow(unused)]
use petgraph::graph::NodeIndex;

use crate::Probability;

use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;

pub struct Trainer(Profile, Tree);

impl Trainer {
    /// i'm making this a static method but in theory we could
    pub fn empty() -> Self {
        Self(Profile::empty(), Tree::empty())
    }
    pub fn train(&mut self, epochs: usize) {
        while self.0.step() < epochs {
            for ref infoset in self.resample() {
                self.0.update_regret(infoset);
                self.0.update_policy(infoset);
            }
        }
        println!("{}", self.0);
    }

    /// the only thing we really need the tree for is to yield infosets for us to sample.
    /// these blocks can be sampled using whatever sampling scheme we like, it's
    /// encapsulated by the Tree itself and how it chooses to unfold from its Nodes.
    fn resample(&mut self) -> Vec<Info> {
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
        let walker = self.0.walker();
        let player = node.player();
        let bucket = node.bucket();
        let mut children = node.datum().spawn();
        if children.is_empty() {
            vec![]
        } else if walker == player {
            children
        } else {
            return children;
            // early return because we need to know
            // how to determinstically sample the same Edge
            // for a given Infoset. something like (Bucket, Epoch)
            // might be unique identifiers for each (Infoset, Iteration)
            let ref mut rng = rand::thread_rng();
            use rand::distributions::Distribution;
            use rand::distributions::WeightedIndex;
            let chance = children
                .iter()
                .map(|(_, edge)| self.0.policy(bucket, edge))
                .collect::<Vec<Probability>>();
            let choice = WeightedIndex::new(chance)
                .expect("at least one > 0")
                .sample(rng);
            let chosen = children.remove(choice);
            vec![chosen]
        }
    }
}
