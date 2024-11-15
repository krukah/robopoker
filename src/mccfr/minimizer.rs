use super::data::Data;
use super::edge::Edge;
use super::info::Info;
use super::node::Node;
use super::partition::Partition;
use super::player::Player;
use super::profile::Profile;
use super::sampler::Sampler;
use super::tree::Branch;
use super::tree::Tree;
use crate::Probability;
use crate::Utility;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;

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
pub struct Solver {
    profile: Profile,
    sampler: Sampler,
    exploring: Vec<Branch>,
}

impl Solver {
    /// load existing profile and encoder from disk
    pub fn load() -> Self {
        Self {
            profile: Profile::load(),
            sampler: Sampler::load(),
            exploring: Vec::new(),
        }
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train(&mut self) {
        let progress = crate::progress(crate::CFR_ITERATIONS);
        while self.profile.next() <= crate::CFR_ITERATIONS {
            for Counterfactual(info, regret, policy) in (0..crate::CFR_BATCH_SIZE)
                .map(|_| self.simulate())
                .collect::<Vec<Tree>>()
                .into_par_iter()
                .map(|t| Partition::from(t))
                .map(|p| Vec::<Info>::from(p))
                .flatten()
                .map(|info| self.counterfactual(info))
                .collect::<Vec<Counterfactual>>()
            {
                let bucket = info.node().bucket();
                self.profile.regret_update(&bucket, &regret.0);
                self.profile.policy_update(&bucket, &policy.0);
            }
            progress.inc(1);
        }
        self.profile.save("blueprint");
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn simulate(&mut self) -> Tree {
        let mut tree = Tree::empty();
        let ref root = tree.insert(self.sampler.root());
        let children = self.sample(root);
        self.exploring = children;
        while let Some(branch) = self.exploring.pop() {
            let ref root = tree.attach(branch);
            let children = self.sample(root);
            self.exploring.extend(children);
        }
        tree
    }

    fn sample(&mut self, node: &Node) -> Vec<Branch> {
        let player = node.player();
        let chance = Player::chance();
        let walker = self.profile.walker();
        let choices = self.branches(node);
        match (choices.len(), player) {
            (0, _) => {
                vec![] //
            }
            (_, p) if p == chance => {
                self.profile.sample_any(choices, node) //
            }
            (_, p) if p != walker => {
                self.profile.witness(node, &choices);
                self.profile.sample_one(choices, node)
            }
            (_, p) if p == walker => {
                self.profile.witness(node, &choices);
                self.profile.sample_all(choices, node)
            }
            _ => panic!("bitches"),
        }
    }

    /// unfiltered set of possible children of a Node,
    /// conditional on its History (# raises, street granularity).
    /// the head Node is attached to the Tree stack-recursively,
    /// while the leaf Data is generated here with help from Sampler.
    /// Rust's ownership makes this a bit awkward but for very good reason!
    /// It has forced me to decouple global (Path) from local (Data)
    /// properties of Tree sampling, which makes lots of sense and is stronger model.
    fn branches(&self, node: &Node) -> Vec<Branch> {
        node.choices()
            .into_iter()
            .map(|e| (e, node.action(e)))
            .map(|(e, a)| (e, node.data().game().apply(a)))
            .map(|(e, g)| (e, g, self.sampler.recall(&g)))
            .map(|(e, g, i)| (e, Data::from((g, i))))
            .map(|(e, d)| (e, d, node.index()))
            .inspect(|(e, d, n)| log::info!("child of {} {} {}", n.index(), e, d.game()))
            .map(|(e, d, n)| Branch(d, e, n))
            .collect()
    }

    /// compute regret and policy vectors for a given infoset
    fn counterfactual(&self, info: Info) -> Counterfactual {
        let regret = Regret(self.profile.regret_vector(&info));
        let policy = Policy(self.profile.policy_vector(&info));
        Counterfactual(info, regret, policy)
    }
}
