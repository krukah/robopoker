//! Lazy tree builder: [`TreeBuilder`] grows game trees incrementally, yielding
//! node indices as they're created.

use crate::*;
use petgraph::graph::NodeIndex;
use std::marker::PhantomData;

/// Lazily builds a game tree, borrowing an [`Encoding`] for state encoding and
/// a [`Solution`] for action sampling.
///
/// Each `next()` pops one pending branch, encodes it, grows the tree, then pushes
/// its `S`-sampled children back onto the stack; iteration ends when the stack
/// drains. Call [`finish()`](Self::finish) afterwards to take the [`Tree`].
pub struct TreeBuilder<'growth, N, P, S>
where
    N: Encoding,
    P: Flow<F = N::F>,
    S: SamplingScheme,
{
    tree: Tree<N::F>,
    todo: Vec<LeafOf<N>>,
    encoder: &'growth N,
    profile: &'growth P,
    sampling: PhantomData<S>,
}

impl<'growth, N, P, S> TreeBuilder<'growth, N, P, S>
where
    N: Encoding,
    P: Flow<F = N::F>,
    S: SamplingScheme,
{
    /// Seeds the tree from `root` and primes the pending stack with its branches.
    ///
    /// `id` is the batch-local tree identifier threaded through to
    /// [`Node::seed`] for RNG seeding.
    pub fn new(encoder: &'growth N, profile: &'growth P, root: GameOf<N>, id: usize) -> Self {
        let mut tree = Tree::new(id);
        let info = encoder.seed(&root);
        let node = tree.seed(info, root);
        let children = S::sample(profile, &node, encoder.branches(&node));
        Self {
            tree,
            todo: children.leaves().collect(),
            encoder,
            profile,
            sampling: PhantomData,
        }
    }

    /// Consumes the builder and returns the tree.
    ///
    /// Only complete once the iterator has returned `None`; called mid-iteration
    /// it yields a partial witness tree.
    pub fn finish(self) -> Tree<N::F> {
        self.tree
    }

    /// Inspect the tree mid-construction without consuming the builder.
    pub fn tree(&self) -> &Tree<N::F> {
        &self.tree
    }

    /// Nodes in the tree so far.
    pub fn len(&self) -> usize {
        self.tree().n()
    }

    pub fn is_empty(&self) -> bool {
        self.tree().n() == 0
    }

    /// Branches still awaiting exploration.
    pub fn pending(&self) -> usize {
        self.todo().len()
    }

    fn tree_mut(&mut self) -> &mut Tree<N::F> {
        &mut self.tree
    }

    fn todo(&self) -> &[LeafOf<N>] {
        &self.todo
    }

    fn todo_mut(&mut self) -> &mut Vec<LeafOf<N>> {
        &mut self.todo
    }

    fn encoder(&self) -> &'growth N {
        self.encoder
    }

    fn profile(&self) -> &'growth P {
        self.profile
    }

    /// Iterate to completion and return the finished tree.
    pub fn build(mut self) -> Tree<N::F> {
        while self.next().is_some() {}
        self.finish()
    }
}

impl<N, P, S> Iterator for TreeBuilder<'_, N, P, S>
where
    N: Encoding,
    P: Flow<F = N::F>,
    S: SamplingScheme,
{
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let (encoder, profile) = (self.encoder(), self.profile());
        let leaf = self.todo_mut().pop()?;
        let terminal = leaf.game().turn().is_terminal();
        // A terminal carries its parent's info: nothing reads it, and encoding it costs a lookup.
        let info = if terminal {
            *self.tree().at(leaf.parent()).info()
        } else {
            encoder.info(self.tree(), leaf)
        };
        debug_assert!(
            terminal || info == encoder.recall(self.tree(), leaf),
            "recall violation: Encoding::info disagreed with Encoding::recall"
        );
        let node = self.tree_mut().grow(info, leaf);
        let index = node.index();
        let children = (!terminal).then(|| S::sample(profile, &node, encoder.branches(&node)));
        self.todo_mut().extend(children.into_iter().flat_map(Branches::leaves));
        Some(index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Lower bound is pending branches, upper bound unknown
        (self.todo().len(), None)
    }
}
