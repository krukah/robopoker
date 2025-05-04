use super::edge::Edge;
use super::encoder::Encoder;
use super::game::Game;
use super::info::Info;
use super::profile::Profile;
use super::turn::Turn;
use crate::mccfr::structs::infoset::InfoSet;
use crate::mccfr::structs::tree::Tree;
use crate::mccfr::types::counterfactual::Counterfactual;

/// given access to a Profile and Encoder,
/// we enapsulate the process of
/// 1) sampling Trees
/// 2) computing Counterfactual vectors at each InfoSet
/// 3) updating the Profile after each Counterfactual batch
/// 4) [optional] apply Discount scheduling to updates
pub trait Blueprint: Send + Sync {
    type T: Turn;
    type E: Edge;
    type G: Game<E = Self::E, T = Self::T>;
    type I: Info<E = Self::E, T = Self::T>;
    type P: Profile<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;
    type S: Encoder<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;

    /// Trains the model by running counterfactual regret minimization iterations.
    /// This is the main training loop that drives strategy optimization.
    fn train();

    /// Returns the number of trees to process in each training batch.
    /// Batching allows for more efficient parallel processing of game trees.
    fn batch_size() -> usize;

    /// Returns the total number of game trees to generate and process during training.
    /// More trees generally leads to better strategy convergence.
    fn tree_count() -> usize;

    /// Returns a reference to the encoder used for converting game states to information sets.
    /// The encoder handles abstraction of game states into trainable buckets.
    fn encoder(&self) -> &Self::S;

    /// Returns a reference to the strategy profile being trained.
    /// The profile tracks accumulated regrets and policies that define the strategy.
    fn profile(&self) -> &Self::P;

    /// Advances the trainer state to the next iteration
    fn advance(&mut self);

    /// Returns a mutable reference to the accumulated regret value for the given infoset/edge pair.
    /// This allows updating the historical regret values that drive strategy updates.
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    /// Returns a mutable reference to the accumulated policy weight for the given infoset/edge pair.
    /// This allows updating the historical action weights that determine the final strategy.
    fn mut_policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    /// Updates trainer state based on regret vectors from Profile.
    fn solve(mut self) -> Self
    where
        Self: Sized,
    {
        let t = Self::iterations();
        log::info!("beginning training loop ({})", t);
        for _ in 0..t {
            for ref update in self.batch() {
                self.update_regret(update);
                self.update_weight(update);
            }
            self.advance();
        }
        self
    }

    /// Updates accumulated regret values for each edge in the counterfactual.
    fn update_regret(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, regret) in cfr.1.iter() {
            let discount = self.discount(Some(self.profile().sum_regret(info, edge)));
            let accumulated = self.profile().sum_regret(info, edge);
            let accumulated = accumulated * discount;
            let accumulated = accumulated + regret;
            let accumulated = accumulated.max(crate::REGRET_MIN);
            *self.mut_regret(info, edge) = accumulated;
        }
    }

    /// Updates accumulated policy weights for each edge in the counterfactual.
    fn update_weight(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, policy) in cfr.2.iter() {
            let discount = self.discount(None);
            let accumulated = self.profile().sum_policy(info, edge);
            let accumulated = accumulated * discount;
            let accumulated = accumulated + policy;
            let accumulated = accumulated.max(crate::POLICY_MIN);
            *self.mut_policy(info, edge) = accumulated;
        }
    }

    /// turn a batch of trees into a batch
    /// of infosets into a batch of counterfactual update vectors.
    ///
    /// this encapsulates the largest unit of "update"
    /// that we can generate in parallel / from immutable reference.
    /// it is unclear from RPS benchmarks if:
    /// - what level to parallelize  .collect().into_par_iter()
    /// - what optimal batch size is given N available CPU cores
    /// - for small batches, whether overhead is worth it to parallelize at all
    ///
    /// it would be nice to do a kind of parameter sweep across
    /// these different settings. i should checkout if criterion supports.
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        (0..Self::batch_size())
            // specify batch size in trait implementation
            .into_par_iter()
            .map(|_| self.tree())
            .collect::<Vec<_>>()
            // partition tree into infosets, and only update one player regrets at a time
            .into_iter()
            .flat_map(|tree| tree.partition().into_values())
            .filter(|infoset| infoset.head().game().turn() == self.profile().walker())
            .collect::<Vec<_>>()
            // calculate CFR vectors (policy, regret) for each infoset
            .into_par_iter()
            .map(|infoset| self.counterfactual(infoset))
            .collect::<Vec<_>>()
    }

    /// generate a single tree by growing it DFS from root to leaves
    ///
    /// starts at the root node and recursively builds the game tree by:
    /// - encoding the current node's information
    /// - sampling valid child branches according to the profile's strategy
    /// - adding sampled branches to a todo list for further expansion
    /// - continues until no more unexpanded leaves remain
    fn tree(&self) -> Tree<Self::T, Self::E, Self::G, Self::I> {
        let mut todo = Vec::new();
        let mut tree = Tree::default();
        let root = self.root();
        let info = self.encoder().seed(&root);
        let node = tree.seed(info, root);
        let children = self.encoder().branches(&node);
        let children = self.profile().explore(&node, children);
        todo.extend(children);
        while let Some(leaf) = todo.pop() {
            let info = self.encoder().info(&tree, leaf);
            let node = tree.grow(info, leaf);
            let children = self.encoder().branches(&node);
            let children = self.profile().explore(&node, children);
            todo.extend(children);
        }
        tree
    }

    /// generate the update vectors at a given [InfoSet]. specifically,
    /// calculate the regret and policy for each action, along with
    /// the associated [Info]
    fn counterfactual(
        &self,
        ref infoset: InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Counterfactual<Self::E, Self::I> {
        (
            infoset.info(),
            self.profile().regret_vector(infoset),
            self.profile().policy_vector(infoset),
        )
    }

    /// Returns the root node of the game.
    /// This is the starting point for tree generation.
    ///
    /// we currently require that root generation is
    /// from Self::G, but that could relax to reference &self: Trainer
    fn root(&self) -> Self::G {
        Self::G::root()
    }

    /// Returns the number of iterations to run the training loop.
    /// This is calculated as the total number of trees to generate
    /// divided by the batch size.
    fn iterations() -> usize {
        Self::tree_count() / Self::batch_size()
    }

    // discount parameterization

    /// Discount parameters for the training process.
    /// These values control how quickly the algorithm converges
    /// and how much weight is given to recent updates versus historical data.
    ///
    /// - `alpha`: Controls the rate at which recent updates are given more weight.
    /// - `omega`: Controls the rate at which historical updates are given more weight.
    /// - `gamma`: Controls the rate at which the discount factor decays over time.
    /// - `period`: Controls the frequency of discount updates.
    fn discount(&self, regret: Option<crate::Utility>) -> f32 {
        match regret {
            None => {
                let g = self.gamma();
                let t = self.profile().epochs() as f32;
                (t / (t + 1.)).powf(g)
            }
            Some(r) => {
                let a = self.alpha();
                let o = self.omega();
                let p = self.period() as f32;
                let t = self.profile().epochs() as f32;
                if t % p != 0. {
                    1.
                } else if r > 0. {
                    let x = (t / p).powf(a);
                    x / (x + 1.)
                } else if r < 0. {
                    let x = (t / p).powf(o);
                    x / (x + 1.)
                } else {
                    let x = t / p;
                    x / (x + 1.)
                }
            }
        }
    }
    fn alpha(&self) -> f32 {
        1.5
    }
    fn omega(&self) -> f32 {
        0.5
    }
    fn gamma(&self) -> f32 {
        1.5
    }
    fn period(&self) -> usize {
        1
    }
}
