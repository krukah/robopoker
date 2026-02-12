//! Turn type for subgame-augmented games.
use crate::*;

/// Turn type for subgame-augmented games.
///
/// Wraps the inner game's turn type and adds subgame-specific phases.
/// During the `MetaGame` phase, the opponent player acts to select
/// which alternative world to enter.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubTurn<T>
where
    T: CfrTurn,
{
    /// Inner game turn (player, chance, or terminal).
    Natural(T),
    /// Subgame phase where opponent (T) selects alternative.
    Adverse(T),
}

impl<T> From<usize> for SubTurn<T>
where
    T: CfrTurn,
{
    fn from(player: usize) -> Self {
        Self::Natural(T::from(player))
    }
}

impl<T> CfrTurn for SubTurn<T>
where
    T: CfrTurn,
{
    fn chance() -> Self {
        Self::Natural(T::chance())
    }
    fn terminal() -> Self {
        Self::Natural(T::terminal())
    }
}
