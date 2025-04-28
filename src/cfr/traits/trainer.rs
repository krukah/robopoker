use crate::cfr::structs::infoset::InfoSet;
use crate::cfr::structs::tree::Tree;
use crate::cfr::traits::edge::Edge;
use crate::cfr::traits::game::Game;
use crate::cfr::traits::info::Info;
use crate::cfr::traits::profile::Profile;
use crate::cfr::traits::sampler::Sampler;
use crate::cfr::traits::turn::Turn;
use crate::cfr::types::counterfactual::Counterfactual;

/// given access to a Profile and Encoder,
/// we enapsulate the process of
/// 1) sampling Trees
/// 2) computing Counterfactual vectors at each InfoSet
/// 3) updating the Profile after each Counterfactual batch
/// 4) [optional] apply Discount scheduling to updates
pub trait Trainer {
    type T: Turn;
    type E: Edge;
    type G: Game<E = Self::E, T = Self::T>;
    type I: Info<E = Self::E, T = Self::T>;
    type P: Profile<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;
    type S: Sampler<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;

    fn encoder(&self) -> &Self::S;
    fn profile(&self) -> &Self::P;
    fn discount(&self, regret: Option<crate::Utility>) -> f32;

    fn advance(&mut self);
    fn regret_mut(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;
    fn policy_mut(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    ///

    /// Updates trainer state based on regret vectors from Profile.
    ///
    /// Several open questions remain about the optimal update strategy:
    ///
    /// 1. Discounting: Should we apply discounting to both regrets and policies?
    ///    Currently we discount both but with different schedules - regrets are
    ///    discounted based on their accumulated value while policies use a simpler
    ///    time-based discount.
    ///
    /// 2. Player Updates: Should we update both players' regrets/policies on every
    ///    iteration? Currently we only update the active player but this may lead
    ///    to slower convergence.
    ///
    /// 3. Accumulation: Should we accumulate both regrets and policies over time?
    ///    The theory suggests accumulating regrets is necessary for convergence,
    ///    but maintaining historical policies may not be required. Currently we
    ///    accumulate both.
    fn solve(&mut self) {
        for _ in 0..crate::CFR_ITERATIONS {
            self.advance();
            for ref update in self.batch() {
                self.update_regret(update);
                self.update_weight(update);
            }
        }
    }
    /// Updates accumulated regret values for each edge in the counterfactual.
    ///
    /// Uncertainty #1: Currently applies regret-based discounting, but unclear if this
    /// is optimal compared to simpler time-based discounting used for policies.
    ///
    /// Uncertainty #2: Only updates regrets for the active player, which may slow convergence
    /// compared to updating both players.
    ///
    /// Uncertainty #3: Theory suggests accumulating regrets is necessary for convergence,
    /// so we maintain historical regret values.
    fn update_regret(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, regret) in cfr.1.iter() {
            let accumlated = self.profile().net_regret(info, edge);
            let discount = self.discount(Some(accumlated));
            *self.regret_mut(info, edge) *= discount;
            *self.regret_mut(info, edge) += regret;
        }
    }

    /// Updates accumulated policy weights for each edge in the counterfactual.
    ///
    /// Uncertainty #1: Currently uses simpler time-based discounting compared to
    /// regret-based discounting used for regrets.
    ///
    /// Uncertainty #2: Only updates policies for the active player, which may slow convergence
    /// compared to updating both players.
    ///
    /// Uncertainty #3: Unclear if maintaining historical policy weights is necessary
    /// for convergence, but we accumulate them anyway.
    fn update_weight(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, policy) in cfr.2.iter() {
            let discount = self.discount(None);
            *self.policy_mut(info, edge) *= discount;
            *self.policy_mut(info, edge) += policy;
        }
    }

    /// LEVEL 4:  turn a bunch of infosets into a bunch of counterfactuals
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        self.partition()
            .iter()
            .map(|infoset| {
                (
                    infoset.info(),
                    self.profile().regret_vector(infoset),
                    self.profile().policy_vector(infoset),
                )
            })
            .collect()
    }
    /// LEVEL 2: turn a bunch of trees into a bunch of infosets
    fn partition(&self) -> Vec<InfoSet<Self::T, Self::E, Self::G, Self::I>> {
        self.forest()
            .into_iter()
            .map(|tree| tree.partition().into_values())
            .flatten()
            .filter(|infoset| infoset.head().game().turn() == self.profile().walker())
            .collect()
    }
    /// LEVEL 1: generate a bunch of trees to be partitioned into InfoSets downstream
    fn forest(&self) -> Vec<Tree<Self::T, Self::E, Self::G, Self::I>> {
        (0..crate::CFR_BATCH_SIZE)
            .map(|_| self.tree())
            .collect::<Vec<_>>()
    }
    /// LEVEL 0: generate a single tree by growing it from root to leaves
    fn tree(&self) -> Tree<Self::T, Self::E, Self::G, Self::I> {
        let mut todo = Vec::new();
        let mut tree = Tree::empty();
        let root = Self::G::root();
        let info = self.encoder().seed(&root);
        let node = tree.seed(info, root);
        let children = self.encoder().grow(&node);
        let children = self.profile().sample(&node, children);
        todo.extend(children);
        while let Some(leaf) = todo.pop() {
            let info = self.encoder().info(&tree, leaf);
            let node = tree.grow(info, leaf);
            let children = self.encoder().grow(&node);
            let children = self.profile().sample(&node, children);
            todo.extend(children);
        }
        tree
    }
}
