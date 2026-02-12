//! Information set for subgame-augmented games.
use super::*;
use crate::*;

/// Information set for subgames.
///
/// Either the meta-game root (world selection) or an inner game info set.
/// The enum structure matches the subgame phases cleanly.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubInfo<I, E>
where
    I: CfrInfo<E = E>,
    E: CfrEdge,
{
    /// Meta-game root: world selection phase.
    Root,
    /// Inner game info set.
    Info(I),
    /// Prefix phase: forced edge from history replay.
    Prefix(I, E),
}

impl<I, E> CfrInfo for SubInfo<I, E>
where
    I: CfrInfo<E = E>,
    E: CfrEdge,
{
    type X = SubPublic<I::X, E>;
    type Y = SubSecret<I::Y>;
    type E = SubEdge<E>;
    type T = SubTurn<I::T>;
    fn public(&self) -> Self::X {
        match self {
            Self::Root => SubPublic::Root,
            Self::Info(i) | Self::Prefix(i, _) => SubPublic::Inner(i.public()),
        }
    }
    fn secret(&self) -> Self::Y {
        match self {
            Self::Root => SubSecret::Root,
            Self::Info(i) | Self::Prefix(i, _) => SubSecret::Inner(i.secret()),
        }
    }
    fn choices(&self) -> Vec<Self::E> {
        match self {
            Self::Root => (0..rbp_core::SUBGAME_ALTS).map(SubEdge::World).collect(),
            Self::Info(i) => i.choices().into_iter().map(SubEdge::Inner).collect(),
            Self::Prefix(_, e) => vec![SubEdge::Inner(*e)],
        }
    }
    fn history(&self) -> Vec<Self::E> {
        match self {
            Self::Root => vec![],
            Self::Info(i) | Self::Prefix(i, _) => {
                i.history().into_iter().map(SubEdge::Inner).collect()
            }
        }
    }
}
