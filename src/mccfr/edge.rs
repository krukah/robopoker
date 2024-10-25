use crate::play::action::Action;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Edge {
    Choice(Action),
    Random,
}

impl From<Action> for Edge {
    fn from(action: Action) -> Self {
        match action {
            Action::Draw(_) | Action::Blind(_) => Self::Random,
            _ => Self::Choice(action),
        }
    }
}

impl From<u32> for Edge {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Random,
            n => Self::Choice(Action::from(n - 1)),
        }
    }
}

impl From<Edge> for u32 {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::Random => 0,
            Edge::Choice(action) => u32::from(action) + 1,
        }
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::action::Action;

    #[test]
    fn bijective_u32() {
        assert!([
            Edge::Random,
            Edge::Choice(Action::Fold),
            Edge::Choice(Action::Check),
            Edge::Choice(Action::Call(100)),
            Edge::Choice(Action::Raise(200)),
            Edge::Choice(Action::Shove(1000)),
        ]
        .into_iter()
        .all(|edge| edge == Edge::from(u32::from(edge))));
    }
}
