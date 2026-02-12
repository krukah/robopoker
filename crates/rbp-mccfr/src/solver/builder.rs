//! Lazy tree builder with iterator-based traversal.
//!
//! [`TreeBuilder`] constructs game trees incrementally, yielding node indices
//! as they're created. This enables lazy evaluation patterns and streaming
//! tree construction.

use crate::*;
use petgraph::graph::NodeIndex;
use std::marker::PhantomData;

/// Lazily builds a game tree by yielding node indices during construction.
///
/// Holds a [`Tree`] internally and borrows an [`Encoder`] and [`Profile`]
/// for state encoding and action sampling. Implements [`Iterator`] to yield
/// [`NodeIndex`] values as nodes are added.
///
/// # Type Parameters
///
/// - `S` — Sampling scheme controlling branch exploration
///
/// # Lifetime
///
/// The `'growth` lifetime binds the builder to its encoder and profile references.
/// The builder cannot outlive the referenced strategy components.
///
/// # Iterator Behavior
///
/// Each `next()` call:
/// 1. Pops a branch from the todo stack
/// 2. Encodes the info using the encoder
/// 3. Grows the tree with the new node
/// 4. Samples child branches using the sampling scheme
/// 5. Extends the todo stack with sampled children
/// 6. Returns the new node's index
///
/// When the todo stack is empty, iteration completes.
///
/// # Completion
///
/// After exhausting the iterator, call [`finish()`](Self::finish) to
/// consume the builder and retrieve the completed [`Tree`].
pub struct TreeBuilder<'growth, T, E, G, I, N, P, S>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    N: Encoder<T = T, E = E, G = G, I = I>,
    P: Profile<T = T, E = E, G = G, I = I>,
    S: SamplingScheme,
{
    tree: Tree<T, E, G, I>,
    todo: Vec<Branch<E, G>>,
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
    N: Encoder<T = T, E = E, G = G, I = I>,
    P: Profile<T = T, E = E, G = G, I = I>,
    S: SamplingScheme,
{
    /// Creates a new tree builder starting from the given root game state.
    ///
    /// Seeds the tree with the root node and initializes the todo stack
    /// with its child branches.
    pub fn new(encoder: &'growth N, profile: &'growth P, root: G) -> Self {
        let mut tree = Tree::default();
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

    /// Consumes the builder and returns the completed tree.
    ///
    /// This should be called after iteration is complete (i.e., after
    /// the iterator returns `None`). Calling it mid-iteration will
    /// return a partial tree.
    pub fn finish(self) -> Tree<T, E, G, I> {
        self.tree
    }

    /// Returns a reference to the tree being built.
    ///
    /// Useful for inspection during construction without consuming the builder.
    pub fn tree(&self) -> &Tree<T, E, G, I> {
        &self.tree
    }

    /// Returns the number of nodes in the tree so far.
    pub fn len(&self) -> usize {
        self.tree.n()
    }

    /// Returns true if no nodes have been added yet.
    pub fn is_empty(&self) -> bool {
        self.tree.n() == 0
    }

    /// Returns the number of pending branches to explore.
    pub fn pending(&self) -> usize {
        self.todo.len()
    }

    /// Exhausts the iterator and returns the completed tree.
    ///
    /// Convenience method that iterates to completion and returns the tree.
    pub fn build(mut self) -> Tree<T, E, G, I> {
        while self.next().is_some() {}
        self.finish()
    }
}

impl<'growth, T, E, G, I, N, P, S> Iterator for TreeBuilder<'growth, T, E, G, I, N, P, S>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    N: Encoder<T = T, E = E, G = G, I = I>,
    P: Profile<T = T, E = E, G = G, I = I>,
    S: SamplingScheme,
{
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let leaf = self.todo.pop()?;
        let info = self.encoder.info(&self.tree, leaf);
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
