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
pub trait CfrEncoder {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;

    /// Attestation that this encoder satisfies [`PerfectRecall`] semantics
    /// strongly enough to enable runtime cross-checking of `info(tree, leaf)`
    /// against `resume(past, head)`.
    ///
    /// Set to `true` by encoders that implement [`EmbeddedHistory`] or
    /// [`PerfectRecall`]. When `true` and `cfg(debug_assertions)` is on,
    /// [`TreeBuilder`] asserts that the two paths produce equal infos at
    /// every expansion, catching silent drift early.
    ///
    /// Defaults to `false` because the check must be explicitly opted into —
    /// encoders that legitimately diverge between the two paths (e.g. holdem's
    /// chip-snapping `apply` fixup) would trip the assertion.
    const CHECK_RECALL: bool = false;

    fn seed(&self, game: &Self::G) -> Self::I;

    /// Creates info set label for a child state during tree expansion.
    ///
    /// The default implementation delegates to [`CfrEncoder::resume`] with an
    /// empty past and the post-application game state. This is correct for
    /// any encoder whose [`CfrInfo`] is a pure function of the game state
    /// alone — see [`EmbeddedHistory`]. Encoders whose info depends on tree
    /// context (e.g. when the `resume` path cannot reconstruct the full
    /// current-phase history from `head` alone) must override this method.
    fn info(&self, _: &Tree<Self::T, Self::E, Self::G, Self::I>, leaf: Leaf<Self::E, Self::G>) -> Self::I {
        let (_, game, _) = leaf;
        self.resume(std::iter::empty(), &game)
    }

    /// Build info from a downward edge sequence and the resulting game head.
    ///
    /// Takes any `IntoIterator<Item = Self::E>` so callers can pass a bare
    /// slice (via `.iter().copied()`), a `Vec` (by value or by `.into_iter()`),
    /// or the edge projection of a [`Descent`](crate::Descent) stream
    /// (via [`JumpStream::edges`](crate::JumpStream)) without an
    /// intermediate collect.
    fn resume<P>(&self, past: P, head: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>;

    /// Replays a path downward from root, yielding `(turn, info, edge)` at
    /// each decision node. Dual of [`Node::decisions`], which walks upward.
    ///
    /// Produces the same `(T, I, E)` triples, enabling reach computations
    /// to consume either direction through the same iterator interface.
    fn replay(&self, root: Self::G, path: impl IntoIterator<Item = Self::E>) -> Vec<(Self::T, Self::I, Self::E)> {
        let mut game = root;
        let mut past: Vec<Self::E> = Vec::new();
        path.into_iter()
            .filter_map(|edge| {
                let turn = game.turn();
                let info = self.resume(past.iter().copied(), &game);
                past.push(edge);
                game = game.apply(edge);
                (!turn.is_chance()).then_some((turn, info, edge))
            })
            .collect()
    }
    /// Delegates branching to the node, which has all the necessary
    /// information to compute the valid edges and resulting game states.
    fn branches(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> Vec<Leaf<Self::E, Self::G>> {
        node.branches()
    }
}

/// Blanket impl allowing references to encoders to be used directly.
impl<N> CfrEncoder for &N
where
    N: CfrEncoder,
{
    type T = N::T;
    type E = N::E;
    type G = N::G;
    type I = N::I;
    const CHECK_RECALL: bool = N::CHECK_RECALL;

    fn seed(&self, game: &Self::G) -> Self::I {
        (*self).seed(game)
    }

    fn info(&self, tree: &Tree<Self::T, Self::E, Self::G, Self::I>, leaf: Leaf<Self::E, Self::G>) -> Self::I {
        (*self).info(tree, leaf)
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        (*self).resume(past, game)
    }

    fn branches(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> Vec<Leaf<Self::E, Self::G>> {
        (*self).branches(node)
    }
}
