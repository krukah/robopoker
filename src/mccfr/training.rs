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
use crate::play::action::Action;
use crate::Probability;
use crate::Utility;
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
    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn sample(&mut self) -> Sample {
        log::info!("sampling tree");
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
        println!("\n{}\n", self.profile);
        println!("\n{}\n", tree);
        Sample(tree, partition)
    }

    /// Process a node: witness it for profile and partition if necessary,
    /// and add its children to the exploration queue.
    fn visit(&mut self, head: &Node, queue: &mut Vec<Branch>, infosets: &mut Partition) {
        let explored = self.explore(head);
        if head.player() == self.profile.walker() {
            infosets.witness(head);
        }
        if head.player() != Player::chance() {
            self.profile.witness(head, &explored);
        }
        for (tail, from) in explored {
            queue.push((tail, from, head.index()));
        }
    }

    /// generate children for a given node
    /// under external sampling rules.
    /// explore all MY options
    /// but only 1 of Chance, 1 of Villain
    fn explore(&self, node: &Node) -> Vec<(Data, Edge)> {
        let children = self.children(node);
        let walker = self.profile.walker();
        let chance = Player::chance();
        let player = node.player();
        if children.is_empty() {
            vec![]
        } else if player == chance {
            self.take_any(children, node)
        } else if player == walker {
            self.take_all(children, node)
        } else if player != walker {
            self.take_one(children, node)
        } else {
            panic!("at the disco")
        }
    }
    fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
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
    fn take_all(&self, choices: Vec<(Data, Edge)>, _: &Node) -> Vec<(Data, Edge)> {
        assert!(choices
            .iter()
            .all(|(_, edge)| matches!(edge, Edge::Choice(_))));
        choices
    }
    /// uniform sampling of chance Edge
    fn take_any(&self, mut choices: Vec<(Data, Edge)>, head: &Node) -> Vec<(Data, Edge)> {
        let ref mut rng = self.profile.rng(head);
        let n = choices.len();
        let choice = rng.gen_range(0..n);
        let chosen = choices.remove(choice);
        assert!(matches!(chosen, (_, Edge::Random)));
        vec![chosen]
    }
    /// Profile-weighted sampling of opponent Edge
    fn take_one(&self, mut choices: Vec<(Data, Edge)>, head: &Node) -> Vec<(Data, Edge)> {
        let ref mut rng = self.profile.rng(head);
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
