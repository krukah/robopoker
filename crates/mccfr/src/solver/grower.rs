use crate::*;

/// An encoder and a profile that grow trees together: what [`Solver`] and any other training
/// loop share before they differ in how they learn from those trees.
pub trait Grower: CfrSpec {
    type N: Encoding<F = Self::F>;
    type P: Flow<F = Self::F>;
    /// Which branches get explored during tree construction.
    type S: SamplingScheme;

    fn encoder(&self) -> &Self::N;

    fn profile(&self) -> &Self::P;

    fn root(&self) -> GameOf<Self> {
        <GameOf<Self>>::root()
    }

    /// `id` is the tree's batch-local identifier (see [`Tree::new`]), so trees within a batch
    /// get distinct, deterministic ids.
    fn tree(&self, id: usize) -> Tree<Self::F> {
        TreeBuilder::<_, _, Self::S>::new(self.encoder(), self.profile(), self.root(), id).build()
    }

    /// Exploitability over a fully-expanded tree, delegated to the Profile.
    fn exploitability(&self) -> pokerkit::Utility {
        self.profile().exploitability(
            TreeBuilder::<_, _, VanillaSampling>::new(
                self.encoder(),
                self.profile(),
                <GameOf<Self>>::exploitability_root(),
                0,
            )
            .build(),
        )
    }

    /// Monte Carlo exploitability estimate over `n` random deals.
    ///
    /// `exploitability_root()` deals fresh each call, so the mean converges to
    /// true expected exploitability at O(1/√n). It is an *upper* bound: per-deal
    /// best response is less constrained than per-infoset (Jensen's inequality).
    fn mxploitability(&self, n: usize) -> pokerkit::Utility {
        (0..n).map(|_| self.exploitability()).sum::<pokerkit::Utility>() / n as pokerkit::Utility
    }
}
