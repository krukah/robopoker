use crate::cfr::bucket::Bucket;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use crate::cfr::profile::Profile;
use crate::Probability;
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
    pub fn infosets(self) -> Vec<Info> {
        // println!("yielding ownership of infosets");
        self.infos.into_values().collect()
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    pub fn dfs(profile: &mut Profile) -> Self {
        // println!("building tree from root");
        let mut tree = Self::empty();
        let root = tree.append(Data::root(), profile);
        let root = tree.graph_mut().add_node(root);
        assert!(root.index() == 0);
        for (leaf, from) in tree.sample(root, profile) {
            tree.sprawl(profile, leaf, from, root);
        }
        tree
    }
    fn sprawl(&mut self, profile: &mut Profile, node: Data, edge: Edge, seed: NodeIndex) {
        // wrap Node Edge NodeIndex into a Child data struct
        let node = self.append(node, profile);
        let node = self.graph_mut().add_node(node);
        let edge = self.graph_mut().add_edge(seed, node, edge);
        assert!(node.index() == edge.index() + 1);
        for (tail, from) in self.sample(node, profile) {
            self.sprawl(profile, tail, from, node);
        }
    }
    fn append(&mut self, data: Data, profile: &mut Profile) -> Node {
        let player = data.player().to_owned();
        let index = NodeIndex::new(self.graph_ref().node_count());
        let graph = self.graph_raw();
        let node = Node::from((index, graph, data));
        if player != Player::Chance {
            self.witness(&node);
            profile.witness(&node);
        }
        node
    }
    fn witness(&mut self, node: &Node) {
        let bucket = node.bucket();
        let index = node.index();
        if let Some(infoset) = self.infos.get_mut(bucket) {
            infoset.push(index);
        } else {
            let graph = self.graph_raw();
            let infoset = Info::from((index, graph));
            self.infos.insert(bucket.clone(), infoset);
        }
    }
    fn empty() -> Self {
        let infos = HashMap::new();
        let graph = Box::new(DiGraph::with_capacity(0, 0));
        Self { infos, graph }
    }
    fn sample(&self, head: NodeIndex, profile: &mut Profile) -> Vec<(Data, Edge)> {
        let player = self.node(head).player();
        let walker = profile.walker();
        if walker == player {
            self.sample_all(head, profile)
        } else {
            self.sample_all(head, profile)
            // self.sample_one(head, profile)
        }
    }
    fn sample_all(&self, head: NodeIndex, _: &mut Profile) -> Vec<(Data, Edge)> {
        self.node(head).datum().spawn()
    }
    fn sample_one(&self, head: NodeIndex, profile: &mut Profile) -> Vec<(Data, Edge)> {
        let ref mut rng = rand::thread_rng();
        let mut children = self.sample_all(head, profile);
        if children.is_empty() {
            return vec![];
        }
        let chance = children
            .iter()
            .map(|(_, edge)| profile.policy(self.node(head).bucket(), edge))
            .collect::<Vec<Probability>>();
        let choice = WeightedIndex::new(chance)
            .expect("same length, at least one > 0")
            .sample(rng);
        let chosen = children.remove(choice);
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;
        vec![chosen]
    }
    fn node(&self, head: NodeIndex) -> &Node {
        self.graph_ref()
            .node_weight(head)
            .expect("being spawned safely in recursion")
    }
    fn graph_ref(&self) -> &DiGraph<Node, Edge> {
        self.graph.as_ref()
    }
    fn graph_mut(&mut self) -> &mut DiGraph<Node, Edge> {
        self.graph.as_mut()
    }
    fn graph_raw(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.graph.as_ref())
    }
}
