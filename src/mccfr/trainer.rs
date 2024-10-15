use super::data::Vertex;
use super::edge::Edge;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::clustering::sampling::Sampler;
use petgraph::graph::NodeIndex;

/// need some async upload/download methods for Profile
/// thesee are totally Tree functions
/// i should hoist INfoSet one level up into this struct
pub struct Explorer {
    tree: Tree,
    profile: Profile,
    sampler: Sampler, // mapping: Abstractor
}

/// impl Iterator<Item = Tree> for Explorer

impl Explorer {
    const EPOCHS: usize = 100_000;
    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets might also come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train() {
        log::info!("training blueprint");
        let ref mut explorer = Self::empty();
        while explorer.profile.next() <= Self::EPOCHS {
            explorer.mcts();
            for ref infoset in explorer.tree.infosets() {
                if explorer.profile.walker() == infoset.node().player() {
                    explorer.profile.update_regret(infoset);
                    explorer.profile.update_policy(infoset);
                }
            }
        }
        log::info!("saving blueprint");
        explorer.profile.save();
    }

    /// i'm making this a static method but in theory we could
    /// download the Profile from disk,
    /// the same way we download the Explorer.
    fn empty() -> Self {
        Self {
            tree: Tree::empty(),
            profile: Profile::empty(),
            sampler: Sampler::download(),
        }
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    /// this is just a base case to handle the root node, presumably a Fn () -> Data.
    /// real-time search implementations may have root nodes provided by the caller.
    fn mcts(&mut self) {
        self.tree = Tree::empty();
        let root = self.root();
        let head = self.witness(root);
        let head = self.tree.graph_mut().add_node(head);
        assert!(head.index() == 0);
        let ref node = self.tree.node(head);
        let ref profile = self.profile;
        for (tail, from) in self.sampler.sample(node, profile) {
            self.dfs(tail, from, head);
        }
    }

    /// recursively build the Tree from the given Node, according to the distribution defined by Profile.
    /// we assert the Tree property of every non-root Node having exactly one parent Edge
    /// we construct the appropriate references in self.attach() to ensure safety.
    fn dfs(&mut self, head: Vertex, edge: Edge, root: NodeIndex) {
        let head = self.witness(head);
        let head = self.tree.graph_mut().add_node(head);
        let edge = self.tree.graph_mut().add_edge(root, head, edge);
        assert!(head.index() == edge.index() + 1);
        let ref node = self.tree.node(head);
        let ref profile = self.profile;
        for (tail, from) in self.sampler.sample(node, profile) {
            self.dfs(tail, from, head);
        }
    }

    /// attach a Node to the Tree,
    /// update the Profile to witness the new Node
    /// update the InfoPartition to witness the new Node.
    fn witness(&mut self, data: Vertex) -> Node {
        let player = data.player().clone();
        let graph = self.tree.graph_ptr();
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
    fn root(&self) -> Vertex {
        use crate::mccfr::bucket::Bucket;
        use crate::play::game::Game;
        let node = Game::root();
        let path = self.sampler.path_abstraction(&Vec::new());
        let abstraction = self.sampler.card_abstraction(&node);
        let bucket = Bucket::from((path, abstraction));
        Vertex::from((node, bucket))
    }
}
