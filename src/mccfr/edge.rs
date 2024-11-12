use crate::mccfr::odds::Odds;
use crate::play::action::Action;
use crate::Chips;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub enum Edge {
    Draw,
    Fold,
    Check,
    Call,
    Raise(Odds),
    Shove,
}

impl Edge {
    pub fn is_shove(&self) -> bool {
        matches!(self, Edge::Shove)
    }
    pub fn is_raise(&self) -> bool {
        matches!(self, Edge::Raise(_))
    }
    pub fn is_chance(&self) -> bool {
        matches!(self, Edge::Draw)
    }
    pub fn is_aggro(&self) -> bool {
        self.is_raise() || self.is_shove()
    }
    pub fn is_choice(&self) -> bool {
        !self.is_chance()
    }
}

impl From<Action> for Edge {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => Edge::Fold,
            Action::Check => Edge::Check,
            Action::Call(_) => Edge::Call,
            Action::Draw(_) => Edge::Draw,
            Action::Shove(_) => Edge::Shove,
            Action::Raise(_) => panic!("raise must be converted from odds"),
            Action::Blind(_) => panic!("blinds are not in any MCCFR trees"),
        }
    }
}
impl From<Odds> for Edge {
    fn from(odds: Odds) -> Self {
        Edge::Raise(odds)
    }
}

/// usize bijection
impl From<Edge> for usize {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::Draw => 0,
            Edge::Fold => 1,
            Edge::Check => 2,
            Edge::Call => 3,
            Edge::Shove => 4,
            Edge::Raise(odds) => {
                5 + Odds::GRID
                    .iter()
                    .position(|&o| o == odds)
                    .expect("invalid odds value") as usize
            }
        }
    }
}
impl From<usize> for Edge {
    fn from(value: usize) -> Self {
        match value {
            0 => Edge::Draw,
            1 => Edge::Fold,
            2 => Edge::Check,
            3 => Edge::Call,
            4 => Edge::Shove,
            5..=14 => Edge::Raise(Odds::GRID[value - 5]),
            _ => unreachable!("invalid edge encoding"),
        }
    }
}

/// u64 bijection
impl From<u64> for Edge {
    fn from(value: u64) -> Self {
        // Use first 3 bits for variant tag
        match value & 0b111 {
            0 => Self::Draw,
            1 => Self::Fold,
            2 => Self::Check,
            3 => Self::Call,
            4 => Self::Raise(Odds(
                ((value >> 3) & 0xFF) as Chips,
                ((value >> 11) & 0xFF) as Chips,
            )),
            5 => Self::Shove,
            _ => unreachable!(),
        }
    }
}
impl From<Edge> for u64 {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::Draw => 0,
            Edge::Fold => 1,
            Edge::Check => 2,
            Edge::Call => 3,
            Edge::Raise(Odds(num, den)) => 4 | ((num as u64) << 3) | ((den as u64) << 11),
            Edge::Shove => 5,
        }
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edge::Draw => write!(f, "────────"),
            Edge::Fold => write!(f, "FOLD    "),
            Edge::Check => write!(f, "CHECK   "),
            Edge::Call => write!(f, "CALL    "),
            Edge::Raise(Odds(num, den)) => write!(f, "RAISE{}:{}", num, den),
            Edge::Shove => write!(f, "SHOVE   "),
        }
    }
}

#[cfg(test)]
mod bijection_tests {
    use super::*;

    #[test]
    fn bijective_usize() {
        let raise = Odds::GRID.map(Edge::Raise);
        let edges = [Edge::Draw, Edge::Fold, Edge::Check, Edge::Call, Edge::Shove];
        assert!(edges
            .into_iter()
            .chain(raise)
            .all(|edge| edge == Edge::from(usize::from(edge))));
    }

    #[test]
    fn bijective_u64() {
        let raise = Odds::GRID.map(Edge::Raise);
        let edges = [Edge::Draw, Edge::Fold, Edge::Check, Edge::Call, Edge::Shove];
        assert!(edges
            .into_iter()
            .chain(raise)
            .all(|edge| edge == Edge::from(u64::from(edge))));
    }
}
