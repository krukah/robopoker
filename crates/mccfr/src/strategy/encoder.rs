use crate::*;

/// Maps game states to information set identifiers.
///
/// The encoder is responsible for the abstraction layer: collapsing
/// the vast game state space into a tractable number of information
/// buckets. Simple games (like RPS) have trivial encoding. Complex
/// games (like NLHE) use learned abstractions, such as from k-means clustering.
///
/// # Methods
///
/// - `seed()` — Creates info for the root game state
/// - `info()` — Creates info for a child state during tree expansion
/// - `grow()` — Creates info from explicit history path (for subgame solving)
/// - `branches()` — Returns valid child branches (default delegates to node)
///
/// # Design Notes
///
/// The encoder has access to the full tree context, enabling path-dependent
/// or probabilistic abstractions if needed.
pub trait Encoder {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;

    fn seed(&self, game: &Self::G) -> Self::I;
    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        leaf: Branch<Self::E, Self::G>,
    ) -> Self::I;

    /// Build info from path and game state.
    ///
    /// Used when constructing info outside of tree traversal (e.g., reach
    /// calculation along a linear edge path).
    fn resume(&self, past: &[Self::E], game: &Self::G) -> Self::I;

    /// because we assume both that Games can
    /// be computed from applying Edges, and that
    /// the Node must have access to its Info set,
    /// we can delegate the branching logic to the Node itself,
    /// which already has all the necessary information to compute
    /// the valid edges and resulting game states.
    fn branches(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        node.branches()
    }
}

/// Blanket impl allowing references to encoders to be used directly.
impl<N> Encoder for &N
where
    N: Encoder,
{
    type T = N::T;
    type E = N::E;
    type G = N::G;
    type I = N::I;
    fn seed(&self, game: &Self::G) -> Self::I {
        (*self).seed(game)
    }
    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        leaf: Branch<Self::E, Self::G>,
    ) -> Self::I {
        (*self).info(tree, leaf)
    }
    fn resume(&self, past: &[Self::E], game: &Self::G) -> Self::I {
        (*self).resume(past, game)
    }
    fn branches(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        (*self).branches(node)
    }
}
