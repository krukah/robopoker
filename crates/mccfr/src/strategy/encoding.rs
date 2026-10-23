use crate::*;

/// The node quotient: maps a node to its information-set label as two projections of the full
/// state. The public half reads the path's abstract edges nearest first, the same shape as
/// [`Approximation::coarsen`]; the secret half reads the state alone.
pub trait Encoding: CfrSpec {
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>;

    /// What the label keeps of the hidden state: the actor's cards for an imperfect-information
    /// solve, every card for an omniscient one, a cluster of either.
    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self>;

    fn encode<P>(&self, game: &GameOf<Self>, past: P) -> InfoOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        InfoOf::<Self>::join(self.public(game, past), self.secret(game))
    }

    /// The tree root's label.
    fn seed(&self, game: &GameOf<Self>) -> InfoOf<Self> {
        self.encode(game, std::iter::empty())
    }

    /// Override only to encode incrementally. [`TreeBuilder`] asserts the two agree in debug
    /// builds.
    fn info(&self, tree: &Tree<Self::F>, leaf: LeafOf<Self>) -> InfoOf<Self> {
        self.recall(tree, leaf)
    }

    /// What [`Self::encode`] assigns `leaf`, walking the tree upward from it.
    fn recall(&self, tree: &Tree<Self::F>, leaf: LeafOf<Self>) -> InfoOf<Self> {
        self.encode(
            &leaf.game(),
            std::iter::once(leaf.edge()).chain(tree.at(leaf.parent()).map(|Ascent(edge, _)| edge)),
        )
    }

    /// Replays a path downward from root, yielding a [`Choice`] at each decision node, as
    /// [`Node::decisions`] does walking upward.
    fn replay<I>(&self, root: GameOf<Self>, path: I) -> Vec<Choice<Self::F>>
    where
        I: IntoIterator<Item = EdgeOf<Self>>,
    {
        path.into_iter()
            .scan((root, Vec::new()), |(game, past), edge| {
                let turn = game.turn();
                let info = self.encode(game, past.iter().rev().copied());
                past.push(edge);
                *game = game.apply(edge);
                Some((!turn.is_chance()).then_some(Choice::new(turn, info, edge)))
            })
            .flatten()
            .collect()
    }

    fn branches(&self, node: &Node<Self::F>) -> Branches<GameOf<Self>> {
        node.branches()
    }
}
