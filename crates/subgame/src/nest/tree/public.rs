//! Public state for off-tree-augmented (nested) subgames.
use crate::*;
use mccfr::*;

/// Public-state wrapper. `Entry` is the one decision point whose menu gets the
/// off-tree action appended; `Augmented` sits below it; everywhere else is `Game`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NestPublic<X, Off> {
    /// A canonical decision point.
    Game(X),
    /// The nesting entry: canonical menu ∪ `{Off(off)}`.
    Entry(X, Off),
    /// A descendant of the entry: canonical menu, kept apart from `Game`.
    Augmented(X),
}

impl<X, Off> NestPublic<X, Off> {
    pub fn inner(&self) -> X
    where
        X: Copy,
    {
        match self {
            Self::Game(x) | Self::Entry(x, _) | Self::Augmented(x) => *x,
        }
    }

    /// The three-way tag: the entry itself, a node below an off-tree edge, or the canonical tree.
    pub fn tag<G>(inner: X, game: &NestGame<G>, sawoff: bool) -> Self
    where
        G: Augmentable<Off = Off>,
    {
        if game.is_entry() {
            Self::Entry(inner, game.off())
        } else if sawoff {
            Self::Augmented(inner)
        } else {
            Self::Game(inner)
        }
    }
}

impl<X, Off> CfrPublic for NestPublic<X, Off>
where
    X: CfrPublic,
    X::G: Augmentable<Off = Off>,
    Off: OffPayload,
{
    type G = NestGame<X::G>;
    fn menu(&self) -> impl Iterator<Item = <Self::G as CfrGame>::E> + use<X, Off> {
        match self {
            Self::Game(x) | Self::Augmented(x) => x.menu().map(NestEdge::Game).collect::<Vec<_>>().into_iter(),
            Self::Entry(x, o) => x
                .menu()
                .map(NestEdge::Game)
                .chain(std::iter::once(NestEdge::Off(*o)))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    fn count(&self) -> usize {
        match self {
            Self::Game(x) | Self::Augmented(x) => x.count(),
            Self::Entry(x, _) => x.count() + 1,
        }
    }
}

impl<I, Off> Unwrap<I> for NestInfo<I, Off>
where
    I: CfrInfo,
    GameOf<I>: Augmentable<Off = Off>,
    Off: OffPayload,
{
    fn inner(&self) -> I {
        I::join(self.public().inner(), self.secret())
    }
}
