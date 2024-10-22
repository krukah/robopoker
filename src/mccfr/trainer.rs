use super::bucket::Bucket;
use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::clustering::encoding::Encoder;
use crate::play::game::Game;
use crate::Probability;
use crate::Utility;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::prelude::Rng;
use std::collections::BTreeMap;

const T: usize = 100_000;

type Regret = BTreeMap<Edge, Utility>;
type Policy = BTreeMap<Edge, Probability>;

struct Update(Bucket, Regret, Policy);
struct Sample(Tree, Partition);

pub struct Blueprint {
    profile: Profile,
    encoder: Encoder,
}

impl Blueprint {
    pub fn load() -> Self {
        Self {
            profile: Profile::load(),
            encoder: Encoder::load(),
        }
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train(&mut self) {
        log::info!("training blueprint");
        while self.profile.next() <= T {
            let Sample(ref tree, ref partition) = self.sample();
            for Update(bucket, regret, policy) in self.update(tree, partition) {
                self.update_regret(&bucket, &regret);
                self.update_policy(&bucket, &policy);
            }
        }
        self.profile.save();
    }
    fn update(&self, tree: &Tree, partition: &Partition) -> Vec<Update> {
        partition
            .0
            .iter()
            .map(|(bucket, info)| self.evaluate(tree, info, bucket))
            .collect()
    }
    fn evaluate(&self, tree: &Tree, info: &Info, bucket: &Bucket) -> Update {
        let regret_vector = self.profile.regret_vector(tree, info);
        let policy_vector = self.profile.policy_vector(tree, info);
        Update(bucket.clone(), regret_vector, policy_vector)
    }
    fn update_regret(&mut self, b: &Bucket, r: &Regret) {
        self.profile.update_regret(b, r);
    }
    fn update_policy(&mut self, b: &Bucket, p: &Policy) {
        self.profile.update_policy(b, p);
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn sample(&mut self) -> Sample {
        let mut partition = Partition::new();
        let mut children = Vec::new();
        let mut tree = Tree::empty();
        let root = self.root();
        let root = tree.insert(root);
        let root = tree.at(root);
        assert!(0 == root.index().index());
        self.profile.witness(root);
        if self.profile.walker() == root.player() {
            partition.witness(root);
        }
        for (tail, from) in self.explore(&root) {
            children.push((tail, from, root.index()));
        }
        while let Some((tail, from, root)) = children.pop() {
            let tail = tree.insert(tail);
            let from = tree.attach(from, tail, root);
            let root = tree.at(tail);
            assert!(1 == root.index().index() - from.index());
            self.profile.witness(root);
            if self.profile.walker() == root.player() {
                partition.witness(root);
            }
            for (tail, from) in self.explore(&root) {
                children.push((tail, from, root.index()));
            }
        }
        Sample(tree, partition)
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
    fn root(&self) -> Data {
        let node = Game::root();
        let path = self.encoder.action_abstraction(&vec![]);
        let info = self.encoder.chance_abstraction(&node);
        let bucket = Bucket::from((path, info));
        Data::from((node, bucket))
    }
    fn explore(&self, node: &Node) -> Vec<(Data, Edge)> {
        let ref mut rng = self.profile.rng(node);
        let mut children = self.encoder.children(node);
        let walker = self.profile.walker();
        let chance = Player::chance();
        let player = node.player();
        if children.is_empty() {
            children
        } else if player == walker {
            children
        } else if player == chance {
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

pub struct Partition(BTreeMap<Bucket, Info>);
impl Partition {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn witness(&mut self, node: Node) {
        self.0
            .entry(node.bucket().clone())
            .or_insert_with(Info::new)
            .add(node.index());
    }
}
