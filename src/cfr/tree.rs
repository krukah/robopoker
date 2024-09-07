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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
/// trees
pub struct Tree {
    graph: Rc<RefCell<DiGraph<Node, Edge>>>,
    infos: HashMap<Bucket, Info>,
}

impl Tree {
    pub fn empty() -> Self {
        let infos = HashMap::new();
        let graph = Rc::new(RefCell::new(DiGraph::with_capacity(0, 0)));
        Self { infos, graph }
    }
    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    pub fn dfs(&mut self, profile: &mut Profile) {
        // println!("building tree from root");
        let root = self.wrap(Data::root());
        let root = self.graph.borrow_mut().add_node(root);
        assert!(root.index() == 0);
        for (leaf, from) in self.sample(root, profile) {
            self.sprawl(profile, leaf, from, root);
        }
        self.bucket();
    }
    pub fn infosets(self) -> Vec<Info> {
        // println!("yielding ownership of infosets");
        self.infos.into_values().collect()
    }
    fn sprawl(&mut self, profile: &mut Profile, node: Data, edge: Edge, seed: NodeIndex) {
        // wrap Node Edge NodeIndex into a Child data struct
        let node = self.wrap(node);
        let node = self.graph.borrow_mut().add_node(node);
        let edge = self.graph.borrow_mut().add_edge(seed, node, edge);
        assert!(node.index() == edge.index() + 1);
        for (tail, from) in self.sample(node, profile) {
            self.sprawl(profile, tail, from, node);
        }
    }
    fn bucket(&mut self) {
        // println!("bucketing tree into infosets");
        for node in self.graph.borrow().node_weights() {
            if node.player() == &Player::Chance {
                continue;
            } else if let Some(info) = self.infos.get_mut(node.bucket()) {
                let index = node.index();
                info.push(index);
            } else {
                let index = node.index();
                let bucket = node.bucket().to_owned();
                let infoset = Info::from((index, self.graph.clone()));
                self.infos.insert(bucket, infoset);
            }
        }
    }

    fn wrap(&self, data: Data) -> Node {
        let graph = self.graph.clone();
        let index = self.graph().node_count();
        let index = NodeIndex::new(index);
        Node::from((index, graph, data))
    }

    /// TODO condition on epoch and node player to decide branching factor in tree unpacking
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
    fn sample_all(&self, head: NodeIndex, profile: &mut Profile) -> Vec<(Data, Edge)> {
        let head = self.node(head);
        let children = head
            .datum()
            .spawn()
            .into_iter()
            .map(|child| child.into())
            .collect::<Vec<(Data, Edge)>>();
        profile.remember(head.bucket().to_owned(), &children);
        children
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
        self.graph()
            .node_weight(head)
            .expect("being spawned safely in recursion")
    }
    fn graph(&self) -> &DiGraph<Node, Edge> {
        unsafe { self.graph.as_ptr().as_ref().expect("valid graph") }
    }
}
