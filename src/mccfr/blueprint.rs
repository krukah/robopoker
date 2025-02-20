use super::counterfactual::Counterfactual;
use super::info::Info;
use super::node::Node;
use super::partition::Partition;
use super::player::Player;
use super::policy::Policy;
use super::profile::Profile;
use super::recall::Recall;
use super::sampler::Encoding;
use super::tree::Branch;
use super::tree::Tree;
use crate::cards::street::Street;
use crate::Arbitrary;
use crate::Save;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

/// this is how we learn the optimal strategy of
/// the abstracted game. with the learned Encoder
/// to abstract all Action and Game objects, we
/// populate and use a Profile to sample Trees, calculate
/// regret and policy updates, then apply the updates to
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
    sampler: Encoding,
}

impl Blueprint {
    /// after training, use the learned Profile to advise
    /// a Spot on how to play.
    fn advise(&self, spot: Recall) -> Policy {
        let bucket = self.sampler.bucket(&spot);
        let policy = self.profile.policy(&bucket);
        Policy::from(policy)
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train() {
        if Self::done(Street::random()) {
            log::info!("skipping regret minimization");
        } else {
            log::info!("starting regret minimization");
            Self::make(Street::random()).solve();
        }
    }

    /// the main training loop.
    fn solve(&mut self) {
        log::info!("beginning training loop");
        let progress = crate::progress(crate::CFR_ITERATIONS);
        while self.profile.next() <= crate::CFR_ITERATIONS {
            for counterfactual in self.simulations() {
                let ref regret = counterfactual.regret();
                let ref policy = counterfactual.policy();
                let ref bucket = counterfactual.info().node().bucket().clone();
                self.profile.add_regret(bucket, regret);
                self.profile.add_policy(bucket, policy);
            }
            progress.inc(1);
            let count = self.profile.size();
            let epoch = self.profile.epochs();
            log::debug!("epochs {:<10} buckets {:<10}", epoch, count);
        }
        self.save();
    }

    /// compute regret and policy updates for a batch of Trees.
    fn simulations(&mut self) -> Vec<Counterfactual> {
        self.forest()
            .into_par_iter()
            .map(Partition::from)
            .map(Vec::<Info>::from)
            .flatten()
            .map(|info| self.profile.counterfactual(info))
            .collect::<Vec<Counterfactual>>()
    }

    /// sample a batch of Trees. mutates because we must
    /// Profile::witness all the decision points of the newly
    /// sample Tree.
    fn forest(&mut self) -> Vec<Tree> {
        (0..crate::CFR_BATCH_SIZE)
            .map(|_| self.search())
            .inspect(|tree| log::trace!("{}", tree))
            .collect::<Vec<Tree>>()
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn search(&mut self) -> Tree {
        let mut tree = Tree::empty(self.profile.walker());
        let ref root = tree.plant(self.sampler.seed());
        let mut todo = self.explore(root);
        while let Some(branch) = todo.pop() {
            let ref node = tree.fork(branch);
            let children = self.explore(node);
            todo.extend(children);
        }
        tree
    }

    /// the Node is already attached to the Tree.
    /// here, we calculate, what Branches
    /// would we like to sample from this Node,
    /// conditional on its History and on our sampling
    /// rules? (i.e. external sampling, probing, full
    /// exploration, etc.)
    fn explore(&mut self, node: &Node) -> Vec<Branch> {
        let chance = Player::chance();
        let walker = self.profile.walker();
        let branches = self.sampler.branches(node);
        match (branches.len(), node.player()) {
            (0, _) => {
                vec![] //
            }
            (_, p) if p == chance => {
                self.profile.explore_any(branches, node) //
            }
            (_, p) if p == walker => {
                self.profile.witness(node, &branches);
                self.profile.explore_all(branches, node)
            }
            (_, p) if p != walker => {
                self.profile.witness(node, &branches);
                self.profile.explore_one(branches, node)
            }
            _ => panic!("at the disco"),
        }
    }
}

impl Save for Blueprint {
    fn name() -> &'static str {
        unreachable!()
    }
    fn save(&self) {
        self.profile.save();
    }
    fn done(street: Street) -> bool {
        Encoding::done(street) && Profile::done(street)
    }
    fn make(street: Street) -> Self {
        Self {
            profile: Profile::default(),
            sampler: Encoding::load(street),
        }
    }
    fn load(street: Street) -> Self {
        Self {
            profile: Profile::load(street),
            sampler: Encoding::load(street),
        }
    }
}
