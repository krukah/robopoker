use crate::*;

/// The abstraction layer: maps game states to information set identifiers,
/// collapsing the state space into a tractable number of buckets. Trivial for
/// RPS; learned k-means clusters for NLHE.
///
/// Methods receive full tree context, so path-dependent or probabilistic
/// abstractions are expressible.
pub trait CfrEncoder {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;

    /// Opts into a debug-only assertion that `info(tree, leaf)` agrees with
    /// `resume(past, head)` at every [`TreeBuilder`] expansion, catching silent
    /// drift early.
    ///
    /// Off by default: encoders that legitimately diverge between the two paths
    /// (e.g. nlhe's chip-snapping `apply` fixup) would trip it.
    const CHECK_RECALL: bool = false;

    fn seed(&self, game: &Self::G) -> Self::I;

    /// Info set label for a child state during tree expansion.
    ///
    /// The default delegates to [`CfrEncoder::resume`] with an empty past, which
    /// is correct only when [`CfrInfo`] is a pure function of the game state —
    /// see [`EmbeddedHistory`]. Encoders whose info needs tree context (because
    /// `head` alone can't reconstruct the current-phase history) must override.
    fn info(&self, _: &Tree<Self::T, Self::E, Self::G, Self::I>, leaf: Leaf<Self::E, Self::G>) -> Self::I {
        let (_, game, _) = leaf;
        self.resume(std::iter::empty(), &game)
    }

    /// Build info from a downward edge sequence and the resulting game head.
    ///
    /// Generic over `IntoIterator` so a slice, a `Vec`, or the edge projection
    /// of a [`Descent`](crate::Descent) stream (via
    /// [`JumpStream::edges`](crate::JumpStream)) all pass without a collect.
    fn resume<P>(&self, past: P, head: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>;

    /// Replays a path downward from root, yielding `(turn, info, edge)` at each
    /// decision node — the same triples as [`Node::decisions`] walking upward,
    /// so reach computations consume either direction identically.
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
    /// Valid child branches; delegated to the node, which knows the legal edges.
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
