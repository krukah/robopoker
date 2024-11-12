use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::partition::Partition;
use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;
use crate::mccfr::sampler::Sampler;
use crate::Probability;
use crate::Utility;
use petgraph::graph::NodeIndex;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;
use std::sync::Arc;

struct Branch(Data, Edge, NodeIndex);
struct Regret(BTreeMap<Edge, Utility>);
struct Policy(BTreeMap<Edge, Probability>);
struct Counterfactual(Info, Regret, Policy);

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
    sampler: Sampler,
}

impl Trainer {
    /// load existing profile and encoder from disk
    pub fn load() -> Self {
        Self {
            profile: Profile::load(),
            sampler: Sampler::load(),
        }
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train(&mut self) {
        log::info!("training blueprint");
        let progress = crate::progress(crate::CFR_ITERATIONS);
        while self.profile.next() <= crate::CFR_ITERATIONS {
            let counterfactuals = (0..crate::CFR_BATCH_SIZE)
                .map(|_| self.sample())
                .collect::<Vec<(Tree, Partition)>>()
                .into_par_iter()
                .map(|(tree, partition)| (Arc::new(tree), partition))
                .map(|(tree, partition)| partition.infos(tree))
                .flatten()
                .map(|info| self.counterfactual(info))
                .collect::<Vec<Counterfactual>>();
            for Counterfactual(info, regret, policy) in counterfactuals {
                self.profile.regret_update(info.node().bucket(), &regret.0);
                self.profile.policy_update(info.node().bucket(), &policy.0);
            }
            progress.inc(1);
        }
        self.profile.save("blueprint");
    }

    /// compute regret and policy vectors for a given infoset
    fn counterfactual(&self, info: Info) -> Counterfactual {
        let regret = Regret(self.profile.regret_vector(&info));
        let policy = Policy(self.profile.policy_vector(&info));
        Counterfactual(info, regret, policy)
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn sample(&mut self) -> (Tree, Partition) {
        let mut tree = Tree::empty();
        let mut partition = Partition::new();
        let ref mut infos = partition;
        let ref mut queue = Vec::new();
        let head = self.sampler.root();
        let head = tree.insert(head);
        let head = tree.at(head);
        self.visit(&head, queue, infos);
        #[allow(unused_variables)]
        while let Some(Branch(tail, from, head)) = queue.pop() {
            let tail = tree.insert(tail);
            let from = tree.extend(tail, from, head);
            let head = tree.at(tail);
            self.visit(&head, queue, infos);
        }
        log::trace!("{}", tree);
        (tree, partition)
    }

    /// Process a node: witness it for profile and partition if necessary,
    /// and add its children to the exploration queue.
    /// under external sampling rules:
    /// - explore ALL my options
    /// - explore 1 of Chance
    /// - explore 1 of Villain
    fn visit(&mut self, head: &Node, queue: &mut Vec<Branch>, partition: &mut Partition) {
        log::trace!("visiting node {}", head);
        let children = self.sampler.children(head);
        let walker = self.profile.walker();
        let chance = Player::chance();
        let player = head.player();
        let sample = if children.is_empty() {
            children
        } else if player == chance {
            self.profile.sample_any(children, head)
        } else if player != walker {
            self.profile.witness(head, &children);
            self.profile.sample_one(children, head)
        } else if player == walker {
            partition.witness(head);
            self.profile.witness(head, &children);
            self.profile.sample_all(children, head)
        } else {
            panic!("at the disco")
        };
        for (tail, from) in sample {
            queue.push(Branch(tail, from, head.index()));
        }
    }
}
