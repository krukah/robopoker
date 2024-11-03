use crate::play::action::Action;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Edge {
    Choice(Action),
    Random,
}

impl Edge {
    pub fn is_delay(&self) -> bool {
        matches!(self, Edge::Choice(Action::Check))
    }
    pub fn is_raise(&self) -> bool {
        if let Edge::Choice(action) = self {
            matches!(action, Action::Raise(_) | Action::Shove(_))
        } else {
            false
        }
    }
    pub fn is_choice(&self) -> bool {
        matches!(self, Edge::Choice(_))
    }
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
        match self {
            Edge::Random => write!(f, "────────"),
            Edge::Choice(action) => match action {
                Action::Fold => write!(f, "FOLD    "),
                Action::Check => write!(f, "CHECK   "),
                Action::Call(x) => write!(f, "CALL  {:<2}", x),
                Action::Raise(x) => write!(f, "RAISE {:<2}", x),
                Action::Shove(x) => write!(f, "SHOVE {:<2}", x),
                Action::Blind(x) => write!(f, "BLIND {:<2}", x),
                Action::Draw(_) => unreachable!(),
            },
        }
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
