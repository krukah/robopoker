use super::counterfactual::Counterfactual;
use super::edge::Edge;
use super::encoder::Encoder;
use super::game::Game;
use super::info::Info;
use super::infoset::InfoSet;
use super::profile::Profile;
use super::tree::Tree;
use super::turn::Turn;

/// given access to a Profile and Encoder,
/// we enapsulate the process of
/// 1) sampling Trees
/// 2) computing Counterfactual vectors at each InfoSet
/// 3) updating the Profile after each Counterfactual batch
/// 4) [optional] apply Discount scheduling to updates
pub trait Solver<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn encoder(&self) -> &impl Encoder<T, E, G, I>;
    fn profile(&self) -> &impl Profile<T, E, G, I>;

    fn regret(&mut self, info: &I, edge: &E) -> &mut f32;
    fn policy(&mut self, info: &I, edge: &E) -> &mut f32;

    fn discount_regret(&self) -> f32;
    fn discount_policy(&self) -> f32;

    ///

    fn solve(&mut self) {
        for _ in 0..crate::CFR_ITERATIONS {
            for ref update in self.batch() {
                self.update_regret(update);
                self.update_policy(update);
            }
        }
    }
    fn update_regret(&mut self, cfr: &Counterfactual<E, I>) {
        let ref info = cfr.0.clone();
        for (edge, regret) in cfr.1.iter() {
            *self.regret(info, edge) *= self.discount_regret();
            *self.regret(info, edge) += regret;
        }
    }
    fn update_policy(&mut self, cfr: &Counterfactual<E, I>) {
        let ref info = cfr.0.clone();
        for (edge, policy) in cfr.2.iter() {
            *self.policy(info, edge) *= self.discount_policy();
            *self.policy(info, edge) += policy;
        }
    }

    /// LEVEL 4:  turn a bunch of infosets into a bunch of counterfactuals
    fn batch(&self) -> Vec<Counterfactual<E, I>> {
        self.infos()
            .into_iter()
            .map(|i| self.counterfactual(&i))
            .collect()
    }
    /// LEVEL 3: transform an infoset into a counterfactual
    fn counterfactual(&self, infoset: &InfoSet<T, E, G, I>) -> Counterfactual<E, I> {
        (
            infoset.info(),
            self.profile().regret_vector(infoset),
            self.profile().policy_vector(infoset),
        )
    }
    /// LEVEL 2: turn a bunch of trees into a bunch of infosets
    fn infos(&self) -> Vec<InfoSet<T, E, G, I>> {
        self.trees()
            .into_iter()
            .map(|tree| tree.partition().into_values())
            .flatten()
            .filter(|infoset| infoset.head().game().turn() == self.profile().walker())
            .collect()
    }
    /// LEVEL 1: generate a bunch of trees to be partitioned into InfoSets downstream
    fn trees(&self) -> Vec<Tree<T, E, G, I>> {
        (0..crate::CFR_BATCH_SIZE)
            .map(|_| self.tree())
            .collect::<Vec<_>>()
    }
    /// LEVEL 0: generate a single tree by growing it from root to leaves
    fn tree(&self) -> Tree<T, E, G, I> {
        let mut todo = Vec::new();
        let mut tree = Tree::<T, E, G, I>::empty();
        let root = G::root();
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
