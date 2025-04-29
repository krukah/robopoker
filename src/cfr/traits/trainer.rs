use super::edge::Edge;
use super::encoder::Encoder;
use super::game::Game;
use super::info::Info;
use super::profile::Profile;
use super::turn::Turn;
use crate::cfr::structs::infoset::InfoSet;
use crate::cfr::structs::tree::Tree;
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
    type S: Encoder<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;

    fn encoder(&self) -> &Self::S;
    fn profile(&self) -> &Self::P;
    fn discount(&self, regret: Option<crate::Utility>) -> f32;

    /// Advances the trainer state to the next iteration
    fn advance(&mut self);

    /// Returns a mutable reference to the accumulated regret value for the given infoset/edge pair.
    /// This allows updating the historical regret values that drive strategy updates.
    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    /// Returns a mutable reference to the accumulated policy weight for the given infoset/edge pair.
    /// This allows updating the historical action weights that determine the final strategy.
    fn policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    ///

    /// Updates trainer state based on regret vectors from Profile.
    fn solve(mut self) -> Self
    where
        Self: Sized,
    {
        log::info!("beginning training loop ({})", crate::CFR_ITERATIONS);
        for _ in 0..crate::CFR_ITERATIONS {
            self.advance();
            for ref update in self.batch() {
                self.update_regret(update);
                self.update_weight(update);
            }
        }
        self
    }
    /// Updates accumulated regret values for each edge in the counterfactual.
    fn update_regret(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, regret) in cfr.1.iter() {
            let discount = self.discount(Some(self.profile().regret(info, edge)));
            *self.regret(info, edge) *= discount;
            *self.regret(info, edge) += regret;
        }
    }

    /// Updates accumulated policy weights for each edge in the counterfactual.
    fn update_weight(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, policy) in cfr.2.iter() {
            let discount = self.discount(None);
            *self.policy(info, edge) *= discount;
            *self.policy(info, edge) += policy;
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
        let mut tree = Tree::default();
        let root = Self::G::root();
        let info = self.encoder().seed(&root);
        let node = tree.seed(info, root);
        let children = self.encoder().branches(&node);
        let children = self.profile().sample(&node, children);
        todo.extend(children);
        while let Some(leaf) = todo.pop() {
            let info = self.encoder().info(&tree, leaf);
            let node = tree.grow(info, leaf);
            let children = self.encoder().branches(&node);
            let children = self.profile().sample(&node, children);
            todo.extend(children);
        }
        tree
    }
}
