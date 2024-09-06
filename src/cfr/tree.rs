use crate::cfr::bucket::Bucket;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::cfr::profile::Profile;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::ptr::NonNull;

/// trees
pub struct Tree {
    graph: Box<DiGraph<Node, Edge>>,
    infos: HashMap<Bucket, Info>,
}

impl Tree {
    pub fn sample(profile: &mut Profile) -> Vec<Info> {
        let mut tree = Self::empty();
        tree.dfs(profile);
        tree.populate();
        tree.infos.into_values().collect()
    }

    /// create an empty Tree
    fn empty() -> Self {
        let infos = HashMap::new();
        let graph = Box::new(DiGraph::with_capacity(0, 0));
        Self { infos, graph }
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    fn dfs(&mut self, profile: &mut Profile) {
        let root = self.wrap(Data::root());
        let root = self.graph.add_node(root);
        for (leaf, from) in self.explore(profile, root) {
            self.recurse(profile, leaf, from, root);
        }
    }

    /// recursively append Data into DiGraph while yielding Nodes for safely unsafe circular reference
    fn recurse(&mut self, profile: &mut Profile, head: Data, edge: Edge, seed: NodeIndex) {
        let head = self.wrap(head);
        let head = self.graph.add_node(head);
        let edge = self.graph.add_edge(seed, head, edge);
        for (tail, from) in self.explore(profile, head) {
            self.recurse(profile, tail, from, head);
        }
        let _ = edge;
    }

    /// populate infoset HashMap with indices of all nodes in the tree
    fn populate(&mut self) {
        for node in self.graph.node_weights() {
            if node.player() == &Player::Chance {
                continue;
            } else if let Some(info) = self.infos.get_mut(node.bucket()) {
                let index = node.index();
                info.push(index);
            } else {
                let index = node.index();
                let bucket = node.bucket().to_owned();
                let infoset = Info::from((index, NonNull::from(self.graph.as_ref())));
                self.infos.insert(bucket, infoset);
            }
        }
    }

    /// create a Node from Data using current boxed self.graph state to safely achieve self-reference
    fn wrap(&self, data: Data) -> Node {
        let graph = NonNull::from(self.graph.as_ref());
        let index = NodeIndex::new(self.graph.node_count());
        Node::from((index, graph, data))
    }

    /// optionally, use external outcome sampling
    /// TODO condition on epoch and node player to decide branching factor in tree unpacking
    fn explore(&self, profile: &mut Profile, head: NodeIndex) -> Vec<(Data, Edge)> {
        self.explore_one(profile, head)
    }

    /// yield all possible children of the node located at head
    /// explores all children of the current node
    /// high branching factor -> exploring all our options
    fn explore_all(&self, head: NodeIndex) -> Vec<(Data, Edge)> {
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
    fn explore_one(&self, profile: &mut Profile, head: NodeIndex) -> Vec<(Data, Edge)> {
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;
        let ref mut rng = rand::thread_rng();
        let mut leaves = self.explore_all(head);
        let chance = leaves
            .iter()
            .map(|(data, edge)| profile.memory(data.bucket().clone(), edge.clone()).advice)
            .collect::<Vec<f32>>();
        let choice = WeightedIndex::new(chance)
            .expect("same length, at least one > 0")
            .sample(rng);
        let chosen = leaves.remove(choice);
        vec![chosen]
    }
}
