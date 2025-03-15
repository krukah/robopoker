use super::data::Data;
use super::edge::Edge;
use super::node::Node;
use super::player::Player;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;

pub struct Leaf(pub Data, pub Edge, pub NodeIndex);
impl Leaf {
    pub fn edge(&self) -> &Edge {
        &self.1
    }
}

/// Represents the game tree structure used in Monte Carlo Counterfactual Regret Minimization (MCCFR).
///
/// The `Tree` struct contains two main components:
/// 1. A directed graph (`DiGraph`) representing the game tree, where nodes are game states and edges are actions.
/// 2. A mapping from `Bucket`s to `Info`sets, which groups similar game states together.
#[derive(Debug, Default)]
pub struct Tree(DiGraph<Data, Edge>, Player);

impl Tree {
    pub fn all(&self) -> Vec<Node> {
        self.0.node_indices().map(|n| self.at(n)).collect()
    }
    pub fn at(&self, index: NodeIndex) -> Node {
        Node::from((index, &self.0))
    }
    pub fn empty(player: Player) -> Self {
        Self(DiGraph::with_capacity(0, 0), player)
    }
    pub fn walker_onlyinpartition(&self) -> Player {
        self.1
    }
    pub fn graph(&self) -> &DiGraph<Data, Edge> {
        &self.0
    }

    /// special insertion logic for the root node
    /// which, without a parent Branch, has slightly different
    /// bucket calculation logic.
    pub fn plant(&mut self, seed: Data) -> Node {
        let i = self.0.add_node(seed);
        let bucket = self.at(i).realize();
        self.0
            .node_weight_mut(i)
            .map(|data| data.assign(bucket))
            .inspect(|_| log::trace!("SEED {}", bucket))
            .expect("root index in tree");
        self.at(i)
    }

    /// attach a Branch to the Tree
    /// assuming that the Node has already been ::inserted()
    pub fn fork(&mut self, branch: Leaf) -> Node {
        let leaf = self.0.add_node(branch.0);
        let edge = branch.1;
        let root = branch.2;
        self.0.add_edge(root, leaf, edge);
        let bucket = self.at(leaf).realize();
        self.0
            .node_weight_mut(leaf)
            .map(|data| data.assign(bucket))
            .inspect(|_| log::trace!("{}", bucket))
            .expect("node index in tree");
        self.at(leaf)
    }

    /// display the Tree in a human-readable format
    /// be careful because it's really big and recursive
    fn show(&self, f: &mut std::fmt::Formatter, x: NodeIndex, prefix: &str) -> std::fmt::Result {
        if x == NodeIndex::new(0) {
            writeln!(f, "\nROOT   {}", self.at(x).bucket())?;
        }
        let mut children = self
            .0
            .neighbors_directed(x, petgraph::Outgoing)
            .collect::<Vec<_>>();
        let n = children.len();
        children.sort();
        for (i, child) in children.into_iter().enumerate() {
            let last = i == n - 1;
            let stem = if last { "└" } else { "├" };
            let gaps = if last { "    " } else { "│   " };
            let node = self.at(child);
            let head = node.bucket();
            let edge = self
                .0
                .edge_weight(self.0.find_edge(x, child).unwrap())
                .unwrap();
            writeln!(f, "{}{}──{} → {}", prefix, stem, edge, head)?;
            self.show(f, child, &format!("{}{}", prefix, gaps))?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.show(f, NodeIndex::new(0), "")
    }
}
