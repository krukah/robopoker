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
use std::ptr::NonNull;
use std::rc::Rc;
/// trees
pub struct Tree {
    graph: Rc<RefCell<DiGraph<Node, Edge>>>,
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
        let bucket = data.bucket().to_owned();
        let player = data.player().to_owned();
        let count = self.graph().node_count();
        let index = NodeIndex::new(count);
        let node = Node::from((index, self.graph.clone(), data));
        if player != Player::Chance {
            if let Some(infoset) = self.infos.get_mut(&bucket) {
                // old bucket
                infoset.push(index);
            } else {
                // new bucket
                let info = Info::from((index, self.graph.clone()));
                let edges = node
                    .datum()
                    .spawn()
                    .into_iter()
                    .map(|child| child.into())
                    .map(|(_, edge)| edge)
                    .collect::<Vec<Edge>>();
                let p = 1. / edges.len() as Probability;
                for action in edges {
                    profile.insert(bucket, action, p);
                }
                self.infos.insert(bucket, info);
            }
        }
        node
    }
    fn empty() -> Self {
        let infos = HashMap::new();
        let graph = Rc::new(RefCell::new(DiGraph::with_capacity(0, 0)));
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
    fn sample_all(&self, head: NodeIndex, profile: &mut Profile) -> Vec<(Data, Edge)> {
        let head = self.node(head);
        let children = head
            .datum()
            .spawn()
            .into_iter()
            .map(|child| child.into())
            .collect::<Vec<(Data, Edge)>>();
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
        unsafe { self.graph.as_ptr().as_ref().expect("non null") }
    }
    fn graph_mut(&mut self) -> &mut DiGraph<Node, Edge> {
        unsafe { self.graph.as_ptr().as_mut().expect("non null") }
    }
    fn graph_raw(&self) -> NonNull<DiGraph<Node, Edge>> {
        unsafe { NonNull::new_unchecked(self.graph.as_ptr()) }
    }
}
