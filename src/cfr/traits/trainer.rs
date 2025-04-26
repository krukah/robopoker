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
    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;
    fn weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    ///

    fn solve(&mut self) {
        for i in 0..crate::CFR_ITERATIONS {
            log::trace!("training iteration {}", i);
            for ref update in self.batch() {
                self.update_regret(update);
                self.update_weight(update);
            }
        }
    }
    fn update_regret(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, regret) in cfr.1.iter() {
            *self.regret(info, edge) *= self.discount(Some(regret.clone()));
            *self.regret(info, edge) += regret;
        }
    }
    fn update_weight(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.0.clone();
        for (edge, weight) in cfr.2.iter() {
            *self.weight(info, edge) *= self.discount(None);
            *self.weight(info, edge) += weight;
        }
        log::info!(""); // Add empty line for better readability
    }

    /// LEVEL 4:  turn a bunch of infosets into a bunch of counterfactuals
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        let batches = self
            .partition()
            .into_iter()
            .map(|ref i| {
                (
                    i.info(),
                    self.profile().regret_vector(i),
                    self.profile().policy_vector(i),
                )
            })
            .collect();

        batches
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
