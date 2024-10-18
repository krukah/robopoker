use super::bucket::Bucket;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::spot::Spot;
use super::tree::Tree;
use crate::clustering::abstractor::Abstractor;
use crate::play::game::Game;
use crate::Probability;
use petgraph::graph::NodeIndex;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::prelude::Rng;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;

const PARALLEL_ITERATIONS: usize = 10;
const TRAINING_ITERATIONS: usize = 100_000;

struct Delta(Bucket, BTreeMap<Edge, f32>, BTreeMap<Edge, f32>);

/// need some async upload/download methods for Profile
/// thesee are totally Tree functions
/// i should hoist INfoSet one level up into this struct
pub struct Optimizer {
    profile: Profile,
    abstractor: Abstractor, // mapping: Abstractor
}

impl Optimizer {
    /// i'm making this a static method but in theory we could
    /// download the Profile from disk,
    /// the same way we download the Explorer.
    pub fn load() -> Self {
        Self {
            profile: Profile::load(),
            abstractor: Abstractor::load(),
        }
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets might also come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train(&mut self) {
        log::info!("training blueprint");
        while self.profile.next() <= TRAINING_ITERATIONS {
            for Delta(bucket, regret, policy) in (0..PARALLEL_ITERATIONS)
                .map(|_| self.mcts())
                .collect::<Vec<Tree>>()
                .into_par_iter()
                .map(|tree| tree.infosets())
                .flatten()
                .filter(|infoset| infoset.node().player() == self.profile.walker())
                .map(|infoset| self.delta(&infoset))
                .inspect(|_| ())
                .collect::<Vec<Delta>>()
            {
                self.profile.update_regret(&bucket, &regret);
                self.profile.update_policy(&bucket, &policy);
            }
        }
        log::info!("saving blueprint");
        self.profile.save();
    }

    /// returns the bucket, and the regret and policy vectors for the given infoset.
    /// this is the & ref step of the parallel update.
    /// we generate all of these in parallel and then aggregate updates in the
    /// "main thread", aka the outer iteration loop.
    /// parallel reads, serial writes! is the way to go imo
    fn delta(&self, infoset: &Info) -> Delta {
        let bucket = infoset.node().bucket().clone();
        let regret_vector = self.profile.regret_vector(infoset);
        let policy_vector = self.profile.policy_vector(infoset);
        Delta(bucket, regret_vector, policy_vector)
    }

    /// so i guess we need to generate the root node here in Trainer
    /// somehow. i'll move ownership around to make it more natural later.
    /// we need the Explorer(Abstractor) to complete the transformation of:
    /// Game::root() -> Observation -> Abstraction
    ///
    /// NOT deterministic, hole cards (from Game) are thread_rng
    fn root(&self) -> Spot {
        let node = Game::root();
        let path = self.abstractor.path_abstraction(&vec![]);
        let info = self.abstractor.card_abstraction(&node);
        let bucket = Bucket::from((path, info));
        Spot::from((node, bucket))
    }

    /// start from root node and allow data.spawn() to recursively, declaratively build the Tree.
    /// in this sense, Data defines the tree implicitly in its spawn() implementation.
    /// this is just a base case to handle the root node, presumably a Fn () -> Data.
    /// real-time search implementations may have root nodes provided by the caller.
    fn mcts(&mut self) -> Tree {
        let mut tree = Tree::empty();
        let root = self.root();
        let head = self.witness(&mut tree, root);
        let head = tree.add_node(head);
        let node = tree.node(head);
        assert!(head.index() == 0);
        for (tail, from) in self.sample(node) {
            self.dfs(&mut tree, tail, from, head);
        }
        tree
    }

    /// recursively build the Tree from the given Node, according to the distribution defined by Profile.
    /// we assert the Tree property of every non-root Node having exactly one parent Edge
    /// we construct the appropriate references in self.attach() to ensure safety.
    fn dfs(&mut self, tree: &mut Tree, head: Spot, edge: Edge, root: NodeIndex) {
        let head = self.witness(tree, head);
        let head = tree.add_node(head);
        let edge = tree.add_edge(root, head, edge);
        let node = tree.node(head);
        assert!(head.index() == edge.index() + 1);
        for (tail, from) in self.sample(node) {
            self.dfs(tree, tail, from, head);
        }
    }

    /// attach a Node to the Tree,
    /// update the Profile to witness the new Node
    /// update the InfoPartition to witness the new Node.
    fn witness(&mut self, tree: &mut Tree, data: Spot) -> Node {
        let player = data.player().clone();
        let graph = tree.graph_arc();
        let count = tree.graph_ref().node_count();
        let index = NodeIndex::new(count);
        let node = Node::from((index, graph, data));
        if player != Player::Chance {
            tree.witness(&node);
            self.profile.witness(&node);
        }
        node
    }

    /// External Sampling:
    /// choose child according to reach probabilities in strategy profile.
    /// on first iteration, this is equivalent to sampling uniformly.
    ///
    /// Walker Sampling:
    /// follow all possible paths toward terminal nodes
    /// when it's the traverser's turn to move
    ///
    /// Chance Sampling:
    /// choose random child uniformly. this is specific to the game of poker,
    /// where each action at chance node/info/buckets is uniformly likely.
    ///
    fn sample(&self, node: &Node) -> Vec<(Spot, Edge)> {
        let player = node.player();
        let mut children = self.abstractor.children(node);
        let ref mut rng = self.profile.rng(node);
        if children.is_empty() {
            vec![]
        } else if player == self.profile.walker() {
            children
        } else if player == Player::chance() {
            let n = children.len();
            let choice = rng.gen_range(0..n);
            let chosen = children.remove(choice);
            vec![chosen]
        } else {
            let policy = children
                .iter()
                .map(|(_, edge)| self.profile.policy(node, edge))
                .collect::<Vec<Probability>>();
            let choice = WeightedIndex::new(policy)
                .expect("at least one policy > 0")
                .sample(rng);
            let chosen = children.remove(choice);
            vec![chosen]
        }
    }
}
