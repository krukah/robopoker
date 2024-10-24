use super::bucket::Bucket;
use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::partition::Partition;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::clustering::encoding::Encoder;
use crate::play::game::Game;
use crate::Probability;
use crate::Utility;
use petgraph::csr::IndexType;
use petgraph::graph::NodeIndex;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::prelude::Rng;
use std::collections::BTreeMap;

const T: usize = 100_000;

type Branch = (Data, Edge, NodeIndex);
type Regret = BTreeMap<Edge, Utility>;
type Policy = BTreeMap<Edge, Probability>;

struct Update(Bucket, Regret, Policy);
struct Sample(Tree, Partition);

#[derive(Default)]
pub struct Blueprint {
    profile: Profile,
    encoder: Encoder,
}

impl Blueprint {
    /// load existing profile and encoder from disk
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
            for Update(bucket, regret, policy) in self.deltas(tree, partition) {
                self.profile.update_regret(&bucket, &regret);
                self.profile.update_policy(&bucket, &policy);
            }
        }
        self.profile.save();
    }

    /// root node of Game has Blinds posted
    fn root(&self) -> Data {
        let node = Game::root();
        let path = self.encoder.action_abstraction(&vec![]);
        let info = self.encoder.chance_abstraction(&node);
        let bucket = Bucket::from((path, info));
        Data::from((node, bucket))
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn sample(&mut self) -> Sample {
        log::info!("sampling tree");
        let mut children = Vec::new();
        let mut infosets = Partition::new();
        let mut tree = Tree::empty();
        let root = self.root();
        let root = tree.insert(root);
        let root = tree.at(root);
        log::debug!("ROOT {}", root.index().index());
        self.attach(&root, &mut children, &mut infosets);
        while let Some((tail, from, head)) = children.pop() {
            let tail = tree.insert(tail);
            let from = tree.attach(from, tail, head);
            let root = tree.at(tail);
            log::debug!(
                "HEAD {} -> {} {:?}",
                head.index().index(),
                from.index().index() + 1,
                root.player()
            );
            self.attach(&root, &mut children, &mut infosets);
        }
        log::debug!("DONE");
        Sample(tree, infosets)
    }

    /// Process a node: witness it for profile and partition if necessary,
    /// and add its children to the exploration queue.
    fn attach(&mut self, node: &Node, children: &mut Vec<Branch>, infosets: &mut Partition) {
        if node.player() != Player::Chance {
            self.profile.witness(*node);
        }
        if node.player() == self.profile.walker() {
            infosets.witness(*node);
        }
        for (tail, from) in self.children(node) {
            children.push((tail, from, node.index()));
        }
    }

    /// generate children for a given node
    /// under external sampling rules.
    /// explore all my options, only one of my opponents'
    fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
        let ref mut rng = rand::thread_rng(); // self.profile.rng(node);
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
        } else if player != walker {
            let policy = children
                .iter()
                .map(|(_, edge)| self.profile.policy(node, edge))
                .collect::<Vec<Probability>>();
            let choice = WeightedIndex::new(policy)
                .expect("at least one policy > 0")
                .sample(rng);
            let chosen = children.remove(choice);
            vec![chosen]
        } else {
            unreachable!()
        }
    }

    /// take all the infosets in the info partition
    /// and compute their regret and policy vectors
    fn deltas(&self, tree: &Tree, partition: &Partition) -> Vec<Update> {
        partition
            .0
            .iter()
            .map(|(bucket, info)| self.delta(tree, info, bucket))
            .collect()
    }

    /// compute regret and policy vectors for a given infoset
    fn delta(&self, tree: &Tree, info: &Info, bucket: &Bucket) -> Update {
        let regret_vector = self.profile.regret_vector(tree, info);
        let policy_vector = self.profile.policy_vector(tree, info);
        Update(bucket.clone(), regret_vector, policy_vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccfr::training::Blueprint;
    use petgraph::graph::NodeIndex;

    #[test]
    #[ignore]
    fn acyclic() {
        let Sample(tree, _) = Blueprint::default().sample();
        assert!(!petgraph::algo::is_cyclic_directed(tree.graph()));
    }

    #[test]
    #[ignore]
    fn nonempty() {
        let Sample(tree, _) = Blueprint::default().sample();
        assert!(0 < tree.graph().node_count());
    }

    #[test]
    #[ignore]
    fn treelike() {
        let Sample(tree, _) = Blueprint::default().sample();
        assert!(tree
            .graph()
            .node_indices()
            .filter(|n| n.index() != 0)
            .all(|n| {
                1 == tree
                    .graph()
                    .neighbors_directed(n, petgraph::Direction::Incoming)
                    .count()
            }));
    }

    #[test]
    #[ignore]
    fn leaves() {
        let Sample(tree, _) = Blueprint::default().sample();
        assert!(
            tree.at(NodeIndex::new(0)).leaves().len()
                == tree
                    .graph()
                    .node_indices()
                    .filter(|&n| tree
                        .graph()
                        .neighbors_directed(n, petgraph::Direction::Outgoing)
                        .count()
                        == 0)
                    .count()
        );
    }
}
