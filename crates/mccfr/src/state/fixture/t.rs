use crate::*;

/// Three-variant turn; the walk turns terminal at depth 3, and depth 2
/// is a chance node so `current_street` has a boundary to find.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum T {
    Choice,
    Chance,
    Terminal,
}

impl From<usize> for T {
    fn from(_: usize) -> Self {
        T::Choice
    }
}

impl CfrTurn for T {
    fn chance() -> Self {
        T::Chance
    }

    fn terminal() -> Self {
        T::Terminal
    }

    fn players() -> usize {
        2
    }
}
