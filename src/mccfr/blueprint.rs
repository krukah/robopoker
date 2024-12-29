use super::counterfactual::Counterfactual;
use super::info::Info;
use super::node::Node;
use super::partition::Partition;
use super::player::Player;
use super::policy::Policy;
use super::profile::Profile;
use super::sampler::Encoding;
use super::spot::Spot;
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
    sampler: Encoding,
}

impl Solver {
    /// after training, use the learned Profile to advise
    /// a Spot on how to play.
    #[allow(unused)]
    fn advise(&self, spot: Spot) -> Policy {
        let bucket = self.sampler.bucket(&spot);
        let policy = self.profile.policy(&bucket);
        let policy = spot.coalesce(policy);
        Policy::from(policy)
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    pub fn train() {
        if Self::done() {
            log::info!("skipping regret minimization");
        } else {
            log::info!("starting regret minimization");
            Self::make(Street::random()).solve();
        }
    }
    /// check (by filename) if a blueprint solver has been saved to disk.
    fn done() -> bool {
        Encoding::done(Street::random()) && Profile::done(Street::random())
    }
    /// the main training loop.
    fn solve(&mut self) {
        log::info!("beginning training loop");
        let progress = crate::progress(crate::CFR_ITERATIONS);
        while self.profile.next() <= crate::CFR_ITERATIONS {
            for counterfactual in self.updates() {
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
    fn updates(&mut self) -> Vec<Counterfactual> {
        self.batch()
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
    fn batch(&mut self) -> Vec<Tree> {
        (0..crate::CFR_BATCH_SIZE)
            .map(|_| self.sample())
            .inspect(|t| log::trace!("{}", t))
            .collect::<Vec<Tree>>()
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn sample(&mut self) -> Tree {
        let mut tree = Tree::empty(self.profile.walker());
        let ref root = tree.insert(self.sampler.root());
        let mut todo = self.explore(root);
        while let Some(branch) = todo.pop() {
            let ref root = tree.attach(branch);
            let children = self.explore(root);
            todo.extend(children);
        }
        tree
    }

    /// could make this more mut so that we can populate Data::partition : Bucket
    /// by using the self.branches() return to inform the set of possible
    /// continuing Edge Actions.
    /// fn explore(&mut self, tree: &mut Tree,node: &Node) -> Vec<Branch> {
    fn explore(&mut self, node: &Node) -> Vec<Branch> {
        let branches = self.sampler.branches(node);
        let walker = self.profile.walker();
        let chance = Player::chance();
        let player = node.player();
        match (branches.len(), player) {
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
            _ => panic!("kyle walker"),
        }
    }
}

impl Save for Solver {
    fn name() -> &'static str {
        unreachable!()
    }
    fn make(street: Street) -> Self {
        Self {
            profile: Profile::default(),
            sampler: Encoding::load(street),
        }
    }
    fn save(&self) {
        self.profile.save();
    }
    fn done(street: Street) -> bool {
        Encoding::done(street) && Profile::done(street)
    }
    fn load(street: Street) -> Self {
        Self {
            sampler: Encoding::load(street),
            profile: Profile::load(street),
        }
    }
}
