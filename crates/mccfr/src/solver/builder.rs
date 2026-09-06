//! Lazy tree builder: [`TreeBuilder`] grows game trees incrementally, yielding
//! node indices as they're created.

use crate::*;
use petgraph::graph::NodeIndex;
use std::marker::PhantomData;

/// Lazily builds a game tree, borrowing an [`CfrEncoder`] for state encoding and
/// a [`CfrSolution`] for action sampling.
///
/// Each `next()` pops one pending branch, encodes it, grows the tree, then pushes
/// its `S`-sampled children back onto the stack; iteration ends when the stack
/// drains. Call [`finish()`](Self::finish) afterwards to take the [`Tree`].
pub struct TreeBuilder<'growth, T, E, G, I, N, P, S>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    N: CfrEncoder<T = T, E = E, G = G, I = I>,
    P: CfrFlow<T = T, E = E, G = G, I = I>,
    S: SamplingScheme,
{
    tree: Tree<T, E, G, I>,
    todo: Vec<Leaf<E, G>>,
    encoder: &'growth N,
    profile: &'growth P,
    sampling: PhantomData<S>,
}

impl<'growth, T, E, G, I, N, P, S> TreeBuilder<'growth, T, E, G, I, N, P, S>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    N: CfrEncoder<T = T, E = E, G = G, I = I>,
    P: CfrFlow<T = T, E = E, G = G, I = I>,
    S: SamplingScheme,
{
    /// Seeds the tree from `root` and primes the todo stack with its branches.
    ///
    /// `id` is the batch-local tree identifier threaded through to
    /// [`Node::seed`] for RNG seeding.
    pub fn new(encoder: &'growth N, profile: &'growth P, root: G, id: usize) -> Self {
        let mut tree = Tree::new(id);
        let info = encoder.seed(&root);
        let node = tree.seed(info, root);
        let children = encoder.branches(&node);
        let children = S::sample(profile, &node, children);
        Self {
            tree,
            todo: children,
            encoder,
            profile,
            sampling: PhantomData,
        }
    }

    /// Consumes the builder and returns the tree.
    ///
    /// Only complete once the iterator has returned `None`; called mid-iteration
    /// it yields a partial witness tree.
    pub fn finish(self) -> Tree<T, E, G, I> {
        self.tree
    }

    /// Inspect the tree mid-construction without consuming the builder.
    pub fn tree(&self) -> &Tree<T, E, G, I> {
        &self.tree
    }

    /// Nodes in the tree so far.
    pub fn len(&self) -> usize {
        self.tree.n()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.n() == 0
    }

    /// Branches still awaiting exploration.
    pub fn pending(&self) -> usize {
        self.todo.len()
    }

    /// Iterate to completion and return the finished tree.
    pub fn build(mut self) -> Tree<T, E, G, I> {
        while self.next().is_some() {}
        self.finish()
    }
}

impl<T, E, G, I, N, P, S> Iterator for TreeBuilder<'_, T, E, G, I, N, P, S>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    N: CfrEncoder<T = T, E = E, G = G, I = I>,
    P: CfrFlow<T = T, E = E, G = G, I = I>,
    S: SamplingScheme,
{
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let leaf = self.todo.pop()?;
        let info = self.encoder.info(&self.tree, leaf);
        #[cfg(debug_assertions)]
        if N::CHECK_RECALL {
            let (edge, game, head) = leaf;
            let mut past = self.tree.at(head).into_iter().map(Jump::edge).collect::<Vec<_>>();
            past.reverse();
            past.push(edge);
            let replayed = self.encoder.resume(past, &game);
            debug_assert!(
                info == replayed,
                "PerfectRecall violation: CfrEncoder::info(tree, leaf) disagreed with CfrEncoder::resume(past, head)"
            );
        }
        let node = self.tree.grow(info, leaf);
        let children = self.encoder.branches(&node);
        let children = S::sample(self.profile, &node, children);
        self.todo.extend(children);
        Some(node.index())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Lower bound is pending branches, upper bound unknown
        (self.todo.len(), None)
    }
}
