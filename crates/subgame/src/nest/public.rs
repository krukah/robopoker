//! Public state for off-tree-augmented (nested) subgames.
use super::*;
use mccfr::*;

/// Public-state wrapper. `Entry` is the one decision point whose menu gets the
/// off-tree action appended; everywhere else is `Game`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NestPublic<X, Off> {
    /// A canonical decision point.
    Game(X),
    /// The nesting entry: canonical menu ∪ `{Off(off)}`.
    Entry(X, Off),
}

impl<X, Off> CfrPublic for NestPublic<X, Off>
where
    X: CfrPublic,
    Off: OffPayload,
{
    type E = NestEdge<X::E, Off>;
    type T = X::T;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<X, Off> {
        match self {
            Self::Game(x) => x.choices().map(NestEdge::Game).collect::<Vec<_>>().into_iter(),
            Self::Entry(x, o) => x
                .choices()
                .map(NestEdge::Game)
                .chain(std::iter::once(NestEdge::Off(*o)))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    fn subgame(&self) -> Vec<Self::E> {
        match self {
            Self::Game(x) | Self::Entry(x, _) => x.subgame().into_iter().map(NestEdge::Game).collect(),
        }
    }
}
