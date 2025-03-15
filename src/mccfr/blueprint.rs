use super::counterfactual::Counterfactual;
use super::encoder::Encoder;
use super::info::InfoSet;
use super::node::Node;
use super::partition::Partition;
use super::player::Player;
use super::policy::Policy;
use super::profile::Profile;
use super::recall::Recall;
use super::tree::Leaf;
use super::tree::Tree;
use crate::cards::street::Street;
use crate::Arbitrary;
use std::sync::Arc;
use std::sync::RwLock;

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
    profile: Arc<RwLock<Profile>>,
    encoder: Encoder,
}

impl Blueprint {
    /// after training, use the learned Profile to advise
    /// a Spot on how to play.
    pub fn policy(&self, recall: &Recall) -> Policy {
        let bucket = self.encoder.bucket(&recall); // this becomes database lookup on recall.game().sweat(), and the Path's are constructed in memory infalliably
        let profile = self.profile.read().unwrap();
        let policy = profile.policy(&bucket); // expand into Result chained calls to database, trying perfect match but weakening index upon every failure
        policy
    }

    /// here's the training loop. infosets might be generated
    /// in parallel later. infosets come pre-filtered
    /// for the traverser. regret and policy updates are
    /// encapsulated by Profile, but we are yet to impose
    /// a learning schedule for regret or policy.
    #[cfg(feature = "native")]
    pub fn train() {
        use crate::save::upload::Table;
        if Self::done(Street::random()) {
            log::info!("resuming regret minimization");
            Self::load(Street::random()).solve(crate::FINE_TRAINING_ITERATIONS);
        } else {
            log::info!("starting regret minimization");
            Self::grow(Street::random()).solve(crate::MAIN_TRAINING_ITERATIONS);
        }
    }

    /// the main training loop.
    #[cfg(feature = "native")]
    fn solve(self, t: usize) -> Self {
        log::info!("beginning training loop");
        use crate::save::upload::Table;
        let progress = crate::progress(t * crate::CFR_BATCH_SIZE);
        for _ in 0..t {
            let counterfactuals = self.simulations();
            let mut profile = self.profile.write().unwrap();
            for counterfactual in counterfactuals {
                let ref regret = counterfactual.regret();
                let ref policy = counterfactual.policy();
                let ref bucket = counterfactual.info().node().bucket().clone();
                profile.add_regret(bucket, regret);
                profile.add_policy(bucket, policy);
                progress.inc(1);
            }
            {
                log::debug!(
                    "epoch {:<10} touched {:<10}",
                    profile.next(),
                    profile.size()
                );
            }
        }
        progress.finish();
        self.profile.read().unwrap().save();
        self
    }

    /// compute regret and policy updates for a batch of Trees.
    #[cfg(feature = "native")]
    fn simulations(&self) -> Vec<Counterfactual> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        (0..crate::CFR_BATCH_SIZE)
            .into_par_iter() // Now we can parallelize the search itself!
            .map(|_| self.tree())
            .inspect(|tree| log::trace!("{}", tree))
            .map(Partition::from)
            .map(Vec::<InfoSet>::from)
            .flatten()
            .map(|info| self.profile.read().unwrap().counterfactual(info))
            .collect::<Vec<Counterfactual>>()
    }

    /// Build the Tree iteratively starting from the root node.
    /// This function uses a stack to simulate recursion and builds the tree in a depth-first manner.
    fn tree(&self) -> Tree {
        let walker = { self.profile.read().unwrap().walker() };
        let mut tree = Tree::empty(walker);
        let ref root = tree.plant(self.encoder.seed());
        let mut todo = self.sample(root);
        while let Some(branch) = todo.pop() {
            let ref node = tree.fork(branch);
            let children = self.sample(node);
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
    fn sample(&self, node: &Node) -> Vec<Leaf> {
        let chance = Player::chance();
        let walker = { self.profile.read().unwrap().walker() };
        let branches = self.encoder.sample(node);
        match (branches.len(), node.player()) {
            (0, _) => vec![],
            (_, p) if p == chance => self.touch_any(branches, node),
            (_, p) if p != walker => self.touch_one(branches, node),
            (_, p) if p == walker => self.touch_all(branches, node),
            _ => panic!("at the disco"),
        }
    }

    fn touch_any(&self, branches: Vec<Leaf>, node: &Node) -> Vec<Leaf> {
        self.profile.read().unwrap().explore_any(branches, node)
    }

    fn touch_all(&self, branches: Vec<Leaf>, node: &Node) -> Vec<Leaf> {
        let _ = { self.profile.write().unwrap().witness(node, &branches) };
        self.profile.read().unwrap().explore_all(branches, node)
    }

    fn touch_one(&self, branches: Vec<Leaf>, node: &Node) -> Vec<Leaf> {
        let _ = { self.profile.write().unwrap().witness(node, &branches) };
        self.profile.read().unwrap().explore_one(branches, node)
    }
}

#[cfg(feature = "native")]
impl crate::save::upload::Table for Blueprint {
    fn done(street: Street) -> bool {
        Profile::done(street) && Encoder::done(street)
    }

    fn save(&self) {
        self.profile.read().unwrap().save();
        self.encoder.save();
    }

    fn grow(_: Street) -> Self {
        // we require an encoder to be trained & loaded
        // but not necessarily a profile
        Self {
            profile: Arc::new(RwLock::new(Profile::default())),
            encoder: Encoder::load(Street::random()),
        }
    }

    fn load(_: Street) -> Self {
        // basically the same as grow but w the expectation
        // that profile is trained & loaded
        Self {
            profile: Arc::new(RwLock::new(Profile::load(Street::random()))),
            encoder: Encoder::load(Street::random()),
        }
    }

    fn name() -> String {
        unimplemented!()
    }

    fn copy() -> String {
        unimplemented!()
    }

    fn creates() -> String {
        unimplemented!()
    }

    fn indices() -> String {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }

    fn sources() -> Vec<String> {
        unimplemented!()
    }
}
