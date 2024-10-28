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
use crate::clustering::layer::Layer;
use crate::play::action::Action;
use crate::Probability;
use crate::Utility;
use indicatif::ProgressBar;
use petgraph::graph::NodeIndex;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::prelude::Rng;
use std::collections::BTreeMap;

const T: usize = 10_000_000;

type Branch = (Data, Edge, NodeIndex);
type Regret = BTreeMap<Edge, Utility>;
type Policy = BTreeMap<Edge, Probability>;

struct Update_Rm_Bucket(Bucket, Regret, Policy); // don't need Bucket. derive from Info
struct Sample(Tree, Partition);

/// this is how we learn the optimal strategy of
/// the abstracted game. with the learned Encoder
/// to abstract all Action and Game objects, we
/// populate and use a Profile to sample Trees, calculate
/// regret and policy updates, then apply the upddates to
/// Profile strategies. it's useful to think about the
/// 3 steps of Exploration, RegretEvaluation, and PolicyUpdate.
///
/// - Tree exploration mutates Profile since it must
/// "witness" all the decision points of the sampled Tree.
/// - Regret & Policy vector evaluations are pure.
/// - Profile updates mutates Profile for obvious reasons.
#[derive(Default)]
pub struct Trainer {
    profile: Profile,
    encoder: Encoder,
}

impl Trainer {
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
        let progress = Layer::progress(T);
        while self.profile.next() <= T {
            // TODO
            // let samples = (0..N)
            // .map(|_| self.sample_fn_of_encoder_ref()) // may need collection for Tree lifetime
            // .map(|(_, info_iter)| info_iter.collect())
            // .flatten() // Vec<Info>
            // .map(|info| self.vectors(info))
            // .collect() : Vec<Update__RegretPolicyVectors>
            let Sample(ref tree, ref info_iter) = self.sample_fn_of_encoder_ref(); // self.profile.sample(encoder) : FnMut(&Profile, &Encoder) -> (Tree, Partition)
            for Update_Rm_Bucket(bucket, regret, policy) in self.deltas(tree, info_iter) {
                self.profile.update_regret(&bucket, &regret);
                self.profile.update_policy(&bucket, &policy);
            }
            progress.inc(1);
        }
        self.profile.save();
    }

    /// take all the infosets in the info partition
    /// and compute their regret and policy vectors
    fn deltas(&self, _tree: &Tree, info_iter: &Partition) -> Vec<Update_Rm_Bucket> {
        info_iter
            .0
            .iter()
            .map(|(_bucket, info)| self.vectors(_tree, info, _bucket))
            .collect()
    }
    /// compute regret and policy vectors for a given infoset
    fn vectors(&self, tree: &Tree, info: &Info, bucket: &Bucket) -> Update_Rm_Bucket {
        let regret_vector = self.profile.regret_vector(tree, info);
        let policy_vector = self.profile.policy_vector(tree, info);
        Update_Rm_Bucket(bucket.clone(), regret_vector, policy_vector)
    }
    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn sample_fn_of_encoder_ref(&mut self) -> Sample {
        let mut tree = Tree::empty();
        let mut partition = Partition::new();
        let ref mut queue = Vec::new();
        let ref mut infos = partition;
        let head = self.encoder.root();
        let head = tree.insert(head);
        let head = tree.at(head);
        self.visit(&head, queue, infos);
        #[allow(unused_variables)]
        while let Some((tail, from, head)) = queue.pop() {
            let tail = tree.insert(tail);
            let from = tree.extend(tail, from, head);
            let head = tree.at(tail);
            self.visit(&head, queue, infos);
        }
        Sample(tree, partition)
    }

    /// Process a node: witness it for profile and partition if necessary,
    /// and add its children to the exploration queue.
    /// under external sampling rules:
    /// - explore ALL my options
    /// - explore 1 of Chance
    /// - explore 1 of Villain
    fn visit(&mut self, head: &Node, queue: &mut Vec<Branch>, infosets: &mut Partition) {
        let children = self.children_fn_of_encoder_ref(head);
        let walker = self.profile.walker();
        let chance = Player::chance();
        let player = head.player();
        let sample = if children.is_empty() {
            children
        } else if player == chance {
            self.sample_any(children, head)
        } else if player != walker {
            self.profile.witness(head, &children);
            self.sample_one(children, head)
        } else if player == walker {
            infosets.witness(head);
            self.profile.witness(head, &children);
            self.sample_all(children, head)
        } else {
            panic!("at the disco")
        };
        for (tail, from) in sample {
            queue.push((tail, from, head.index()));
        }
    }

    fn children_fn_of_encoder_ref(&self, node: &Node) -> Vec<(Data, Edge)> {
        const MAX_N_RAISE: usize = 2;
        let ref past = node.history();
        let ref game = node.data().game();
        let children = game
            .children()
            .into_iter()
            .map(|(g, a)| self.encoder.encode(g, a, past))
            .collect::<Vec<(Data, Edge)>>();
        if MAX_N_RAISE
            > past
                .iter()
                .rev()
                .take_while(|e| matches!(e, Edge::Choice(_)))
                .count()
        {
            children
        } else {
            children
                .into_iter()
                .filter(|(_, e)| !matches!(e, Edge::Choice(Action::Raise(_))))
                .collect()
        }
    }

    // external sampling

    /// full exploration of my decision space Edges
    fn sample_all(&self, choices: Vec<(Data, Edge)>, _: &Node) -> Vec<(Data, Edge)> {
        assert!(choices
            .iter()
            .all(|(_, edge)| matches!(edge, Edge::Choice(_))));
        choices
    }
    /// uniform sampling of chance Edge
    fn sample_any(&self, choices: Vec<(Data, Edge)>, head: &Node) -> Vec<(Data, Edge)> {
        let ref mut rng = self.profile.rng(head);
        let mut choices = choices;
        let n = choices.len();
        let choice = rng.gen_range(0..n);
        let chosen = choices.remove(choice);
        assert!(matches!(chosen, (_, Edge::Random)));
        vec![chosen]
    }
    /// Profile-weighted sampling of opponent Edge
    fn sample_one(&self, choices: Vec<(Data, Edge)>, head: &Node) -> Vec<(Data, Edge)> {
        let ref mut rng = self.profile.rng(head);
        let mut choices = choices;
        let policy = choices
            .iter()
            .map(|(_, edge)| self.profile.policy(head, edge))
            .collect::<Vec<Probability>>();
        let choice = WeightedIndex::new(policy)
            .expect("at least one policy > 0")
            .sample(rng);
        let chosen = choices.remove(choice);
        assert!(matches!(chosen, (_, Edge::Choice(_))));
        vec![chosen]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccfr::minimizer::Trainer;
    use petgraph::graph::NodeIndex;

    #[test]
    #[ignore]
    fn acyclic() {
        let Sample(tree, _) = Trainer::default().sample_fn_of_encoder_ref();
        assert!(!petgraph::algo::is_cyclic_directed(tree.graph()));
    }

    #[test]
    #[ignore]
    fn nonempty() {
        let Sample(tree, _) = Trainer::default().sample_fn_of_encoder_ref();
        assert!(0 < tree.graph().node_count());
    }

    #[test]
    #[ignore]
    fn treelike() {
        let Sample(tree, _) = Trainer::default().sample_fn_of_encoder_ref();
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
        let Sample(tree, _) = Trainer::default().sample_fn_of_encoder_ref();
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
