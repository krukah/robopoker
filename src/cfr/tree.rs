use crate::cfr::bucket::Bucket;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::ptr::NonNull;

use super::profile::Profile;

/// trees
pub struct Tree {
    graph: Box<DiGraph<Node, Edge>>,
    infos: HashMap<Bucket, Info>,
}

// fn bucketize(&mut self) {
//     for node in self.graph.node_weights() {
//         let index = node.index();
//         let player = node.player();
//         let bucket = node.bucket();
//         if player == &Player::Chance {
//             continue;
//         } else {
//             match self.infos.get_mut(bucket) {
//                 Some(info) => info.push(index),
//                 None => {
//                     let info = Info::from((index, self.graph()));
//                     let bucket = bucket.to_owned();
//                     self.infos.insert(bucket, info);
//                 }
//             }
//         }
//     }
// }

impl Tree {
    pub fn blocks(&self) -> Vec<Info> {
        todo!()
    }
    pub fn new() -> Self {
        let mut tree = Self::empty();
        let ref profile = Profile::new();
        tree.dfs(profile);
        tree
    }

    /// create an empty Tree
    fn empty() -> Self {
        Self {
            infos: HashMap::new(),
            graph: Box::new(DiGraph::with_capacity(0, 0)),
        }
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    fn dfs(&mut self, strat: &Profile) {
        let root = self.wrap(Data::root());
        let root = self.graph.add_node(root);
        for (leaf, from) in self.sample(strat, root) {
            self.dfr(strat, leaf, from, root);
        }
    }

    /// recursively append Data into DiGraph while yielding Nodes for safely unsafe circular reference
    fn dfr(&mut self, strat: &Profile, head: Data, edge: Edge, seed: NodeIndex) {
        let head = self.wrap(head);
        let head = self.graph.add_node(head);
        let edge = self.graph.add_edge(seed, head, edge);
        for (tail, from) in self.sample(strat, head) {
            self.dfr(strat, tail, from, head);
        }
        let _ = edge;
    }

    /// optionally, use external outcome sampling
    /// TODO condition on epoch and node player to decide branching factor in tree unpacking
    fn sample(&self, strat: &Profile, head: NodeIndex) -> Vec<(Data, Edge)> {
        self.sample_one(strat, head)
    }

    /// yield all possible children of the node located at head
    /// explores all children of the current node
    /// high branching factor -> exploring all our options
    fn sample_all(&self, head: NodeIndex) -> Vec<(Data, Edge)> {
        self.graph
            .node_weight(head)
            .expect("being spawned safely in recursion")
            .datum()
            .spawn()
            .into_iter()
            .map(|child| child.into())
            .collect()
    }

    /// choose one of the children according to profile distribution
    /// explores a single randomly selected child
    /// low branching factor -> prevent compinatoric explosion.
    fn sample_one(&self, strat: &Profile, head: NodeIndex) -> Vec<(Data, Edge)> {
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;
        let ref mut rng = rand::thread_rng();
        let mut leaves = self.sample_all(head);
        let chance = leaves
            .iter()
            .map(|&(ref data, ref edge)| strat.get(data.bucket(), edge))
            .collect::<Vec<f32>>();
        let choice = WeightedIndex::new(chance)
            .expect("same length, at least one > 0")
            .sample(rng);
        let chosen = leaves.remove(choice);
        vec![chosen]
    }

    /// create a Node from Data using current boxed self.graph state to safely achieve self-reference
    fn wrap(&self, data: Data) -> Node {
        let graph = NonNull::from(self.graph.as_ref());
        let index = NodeIndex::new(self.graph.node_count());
        Node::from((index, graph, data))
    }
}
