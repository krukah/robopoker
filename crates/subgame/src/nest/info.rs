//! Information set for off-tree-augmented (nested) subgames.
use super::*;
use mccfr::*;

/// Three-way tag. Because a game's current-street info key typically drops
/// street-boundary-crossed history, a hero node reached *through* the off-tree
/// edge and one reached through a canonical action of similar magnitude can
/// hash identically. Keeping `Augmented` (descendant of the off-tree edge)
/// distinct from `Game` (descendant of canonical edges) stops their regrets
/// commingling. See `docs/active/off-tree-nesting.md` insight #2.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NestInfo<I, Off> {
    /// The entry node itself, carrying the spliced off-tree payload.
    Entry(I, Off),
    /// A descendant of the off-tree edge.
    Augmented(I),
    /// A descendant of canonical edges (the common case).
    Game(I),
}

impl<I, Off> NestInfo<I, Off>
where
    I: Copy,
{
    /// The wrapped canonical info, regardless of tag.
    pub fn inner(&self) -> I {
        match self {
            Self::Entry(i, _) | Self::Augmented(i) | Self::Game(i) => *i,
        }
    }
}

impl<I, Off> CfrInfo for NestInfo<I, Off>
where
    I: CfrInfo,
    Off: OffPayload,
{
    type E = NestEdge<I::E, Off>;
    type T = I::T;
    type X = NestPublic<I::X, Off>;
    type Y = I::Y;

    fn public(&self) -> Self::X {
        match self {
            Self::Entry(i, o) => NestPublic::Entry(i.public(), *o),
            Self::Augmented(i) | Self::Game(i) => NestPublic::Game(i.public()),
        }
    }

    fn secret(&self) -> Self::Y {
        self.inner().secret()
    }
}
