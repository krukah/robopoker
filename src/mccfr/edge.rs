use crate::clustering::encoding::Odds;
use crate::play::action::Action;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq)]
pub enum Edge {
    Choice(Action),
    Raises(Odds),
    Random,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Edge::Random, Edge::Random) => true,
            (Edge::Raises(o1), Edge::Raises(o2)) => o1 == o2,
            (Edge::Choice(a1), Edge::Choice(a2)) => {
                std::mem::discriminant(a1) == std::mem::discriminant(a2)
            }
            _ => false,
        }
    }
}

impl Edge {
    pub fn is_raise(&self) -> bool {
        matches!(self, Edge::Raises(_))
    }
    pub fn is_choice(&self) -> bool {
        matches!(self, Edge::Raises(_) | Edge::Choice(_))
    }
    pub fn is_aggro(&self) -> bool {
        matches!(self, Edge::Raises(_) | Edge::Choice(Action::Shove(_)))
    }
    pub fn is_shove(&self) -> bool {
        matches!(self, Edge::Choice(Action::Shove(_)))
    }
    pub fn is_random(&self) -> bool {
        matches!(self, Edge::Random)
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

impl From<u64> for Edge {
    fn from(value: u64) -> Self {
        // Use first 2 bits for variant tag
        match value & 0b11 {
            0 => Self::Random,
            1 => Self::Choice(Action::from((value >> 2) as u32)),
            2 => {
                // Extract numerator and denominator from next 16 bits
                let num = ((value >> 2) & 0xFF) as u8;
                let den = ((value >> 10) & 0xFF) as u8;
                Self::Raises(Odds(num, den))
            }
            _ => unreachable!(),
        }
    }
}

impl From<Edge> for u64 {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::Random => 0,
            Edge::Choice(action) => 1 | ((u64::from(u32::from(action))) << 2),
            Edge::Raises(Odds(num, den)) => 2 | ((num as u64) << 2) | ((den as u64) << 10),
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
            Edge::Raises(odds) => write!(f, "RAISE {}/{}", odds.0, odds.1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::action::Action;

    #[test]
    fn bijective_u64() {
        assert!([
            Edge::Random,
            Edge::Choice(Action::Fold),
            Edge::Choice(Action::Check),
            Edge::Choice(Action::Call(100)),
            Edge::Choice(Action::Raise(200)),
            Edge::Choice(Action::Shove(1000)),
            Edge::Raises(Odds(1, 2)),
            Edge::Raises(Odds(3, 4)),
        ]
        .into_iter()
        .all(|edge| edge == Edge::from(u64::from(edge))));
    }
}
