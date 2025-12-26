use super::*;
use crate::mccfr::*;

/// A Blueprint wrapper that uses caching for efficient regret computation.
/// Delegates all methods to the inner Blueprint except batch(), which uses
/// CachedTree to share cache across infosets from the same tree.
pub struct CachedBlueprint<B: Blueprint> {
    inner: B,
}

impl<B: Blueprint> CachedBlueprint<B> {
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
    pub fn into_inner(self) -> B {
        self.inner
    }
}

impl<B: Blueprint> Blueprint for CachedBlueprint<B> {
    type T = B::T;
    type E = B::E;
    type G = B::G;
    type I = B::I;
    type P = B::P;
    type S = B::S;
    fn batch_size() -> usize {
        B::batch_size()
    }
    fn tree_count() -> usize {
        B::tree_count()
    }
    fn encoder(&self) -> &Self::S {
        self.inner.encoder()
    }
    fn profile(&self) -> &Self::P {
        self.inner.profile()
    }
    fn advance(&mut self) {
        self.inner.advance()
    }
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        self.inner.mut_regret(info, edge)
    }
    fn mut_policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        self.inner.mut_policy(info, edge)
    }
    #[cfg(feature = "server")]
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        (0..Self::batch_size())
            .into_par_iter()
            .map(|_| self.tree())
            .collect::<Vec<_>>()
            .into_iter()
            .inspect(|t| self.inc_nodes(t.n()))
            .flat_map(|tree| CachedTree::new(tree, self.profile()).counterfactuals())
            .collect()
    }
    #[cfg(not(feature = "server"))]
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        (0..Self::batch_size())
            .into_iter()
            .map(|_| self.tree())
            .inspect(|t| self.inc_nodes(t.n()))
            .flat_map(|tree| CachedTree::new(tree, self.profile()).counterfactuals())
            .collect()
    }
}
