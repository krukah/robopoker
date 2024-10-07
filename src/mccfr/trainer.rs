use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::clustering::explorer::Explorer;
use petgraph::graph::NodeIndex;

/// need some async upload/download methods for Profile
pub struct Blueprint {
    explorer: Explorer,
    profile: Profile,
    tree: Tree,
}

impl Blueprint {
    /// i'm making this a static method but in theory we could
    /// download the Profile from disk,
    /// the same way we download the Explorer.
    fn empty() -> Self {
        Self {
            explorer: Explorer::download(),
            profile: Profile::empty(),
            tree: Tree::empty(),
        }
    }
    pub fn train(epochs: usize) {
        let mut this = Self::empty();
        while this.profile.step() <= epochs {
            for ref infoset in this.blocks() {
                if this.profile.walker() == infoset.node().player() {
                    this.profile.update_regret(infoset);
                    this.profile.update_policy(infoset);
                }
            }
        }
    }

    /// the only thing we really need the tree for is to yield infosets for us to sample.
    /// these blocks can be sampled using whatever sampling scheme we like, it's
    /// encapsulated by the Tree itself and how it chooses to unfold from its Nodes.
    fn blocks(&mut self) -> Vec<Info> {
        self.tree = Tree::empty();
        self.dfs();
        self.tree.infosets()
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    /// this is just a base case to handle the root node, presumably a Fn () -> Data.
    /// real-time search implementations may have root nodes provided by the caller.
    fn dfs(&mut self) {
        let root = self.root();
        let head = self.attach(root);
        let head = self.tree.graph_mut().add_node(head);
        let ref node = self.tree.node(head);
        let ref profile = self.profile;
        for (tail, from) in self.explorer.sample(node, profile) {
            self.unfold(tail, from, head);
        }
        assert!(head.index() == 0);
    }

    /// recursively build the Tree from the given Node, according to the distribution defined by Profile.
    /// we assert the Tree property of every non-root Node having exactly one parent Edge
    /// we construct the appropriate references in self.attach() to ensure safety.
    fn unfold(&mut self, head: Data, edge: Edge, root: NodeIndex) {
        let head = self.attach(head);
        let head = self.tree.graph_mut().add_node(head);
        let edge = self.tree.graph_mut().add_edge(root, head, edge);
        let ref node = self.tree.node(head);
        let ref profile = self.profile;
        for (tail, from) in self.explorer.sample(node, profile) {
            self.unfold(tail, from, head);
        }
        assert!(head.index() == edge.index() + 1);
    }

    /// attach a Node to the Tree,
    /// update the Profile to witness the new Node
    /// update the Tree to witness the new Node.
    fn attach(&mut self, data: Data) -> Node {
        let player = data.player().clone();
        let graph = self.tree.graph_raw();
        let count = self.tree.graph_ref().node_count();
        let index = NodeIndex::new(count);
        let node = Node::from((index, graph, data));
        if player != Player::Chance {
            self.profile.witness(&node);
            self.tree.witness(&node);
        }
        node
    }

    /// so i guess we need to generate the root node here in Trainer
    /// somehow. i'll move ownership around to make it more natural later.
    /// we need the Explorer(Abstractor) to complete the transformation of:
    /// Game::root() -> Observation -> Abstraction
    ///
    /// NOT deterministic, hole cards are thread_rng
    fn root(&self) -> Data {
        use crate::mccfr::bucket::Bucket;
        use crate::play::game::Game;
        let node = Game::root();
        let path = self.explorer.path_abstraction(&Vec::new());
        let abstraction = self.explorer.card_abstraction(&node);
        let bucket = Bucket::from((path, abstraction));
        Data::from((node, bucket))
    }
}

impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Trainer profile:\n{}", self.profile)
    }
}
