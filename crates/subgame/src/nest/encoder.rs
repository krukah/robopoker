//! CfrEncoder for off-tree-augmented (nested) subgames.
//!
//! Wraps a base [`CfrEncoder`] whose game is [`Augmentable`] and lifts its info
//! into the three-way [`NestInfo`]. This is where both correctness insights are
//! made concrete (see `docs/active/off-tree-nesting.md`):
//!
//! - **#1 (entry detection):** entry-ness is read off the *game*
//!   ([`NestGame::is_entry`]), not the path.
//! - **#2 (three-way tag):** off-tree-branch descendants tag `Augmented`,
//!   canonical descendants tag `Game`, keeping their regrets separate.
//!
//! The menu splice is automatic: `CfrEncoder::branches` defaults to
//! `node.branches()`, which enumerates `info.public().choices()` — and an
//! `Entry` info's public is [`NestPublic::Entry`], whose `choices()` appends
//! the off-tree edge.
use super::*;
use mccfr::*;

type Off<N> = <<N as CfrEncoder>::G as Augmentable>::Off;
type NestTree<N> = Tree<
    <N as CfrEncoder>::T,
    NestEdge<<N as CfrEncoder>::E, Off<N>>,
    NestGame<<N as CfrEncoder>::G>,
    NestInfo<<N as CfrEncoder>::I, Off<N>>,
>;

/// Wraps `&N`, lifting its info output into [`NestInfo`].
pub struct NestEncoder<'blueprint, N>
where
    N: CfrEncoder,
    N::G: Augmentable,
{
    inner: &'blueprint N,
}

impl<'blueprint, N> NestEncoder<'blueprint, N>
where
    N: CfrEncoder,
    N::G: Augmentable,
{
    pub fn new(inner: &'blueprint N) -> Self {
        Self { inner }
    }
    /// Build the inner canonical info from a nest path (dropping the off-tree
    /// marker, which has no canonical analog), then apply the three-way tag.
    fn classify(&self, path: Vec<NestEdge<N::E, Off<N>>>, game: &NestGame<N::G>) -> NestInfo<N::I, Off<N>> {
        let sawoff = path.iter().any(|e| e.offtree().is_some());
        let inner = self
            .inner
            .resume(path.into_iter().filter_map(NestEdge::game), game.inner());
        tag(inner, game, sawoff)
    }
}

/// The three-way tag rule, kept pure so it is unit-testable without a
/// populated encoder.
pub fn tag<I, G>(inner: I, game: &NestGame<G>, sawoff: bool) -> NestInfo<I, G::Off>
where
    G: Augmentable,
{
    if game.is_entry() {
        NestInfo::Entry(inner, game.off())
    } else if sawoff {
        NestInfo::Augmented(inner)
    } else {
        NestInfo::Game(inner)
    }
}

impl<N> CfrEncoder for NestEncoder<'_, N>
where
    N: CfrEncoder,
    N::G: Augmentable,
{
    type T = N::T;
    type E = NestEdge<N::E, Off<N>>;
    type G = NestGame<N::G>;
    type I = NestInfo<N::I, Off<N>>;

    fn seed(&self, game: &Self::G) -> Self::I {
        self.classify(Vec::new(), game)
    }

    fn info(&self, tree: &NestTree<N>, (edge, game, parent): Leaf<Self::E, Self::G>) -> Self::I {
        let mut path = tree.at(parent).map(mccfr::Jump::edge).collect::<Vec<_>>();
        path.reverse();
        path.push(edge);
        self.classify(path, &game)
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        self.classify(past.into_iter().collect(), game)
    }
}
