use super::edge::Edge;
use super::player::Player;
use crate::play::ply::Ply;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

impl From<Vec<Edge>> for Path {
    fn from(edges: Vec<Edge>) -> Self {
        let depth = edges.len();
        let raise = edges.iter().filter(|e| e.is_raise()).count();
        Path::from((depth, raise))
    }
}

impl From<(usize, usize)> for Path {
    fn from((depth, raise): (usize, usize)) -> Self {
        Path((depth | raise << 32) as u64)
    }
}

impl From<u64> for Path {
    fn from(value: u64) -> Self {
        Path(value)
    }
}

impl From<Path> for u64 {
    fn from(path: Path) -> Self {
        path.0
    }
}

impl Path {
    fn depth(&self) -> usize {
        (self.0 & 0xFFFFFFFF) as usize
    }

    fn raises(&self) -> usize {
        (self.0 >> 32) as usize
    }
}

impl From<Path> for Player {
    fn from(path: Path) -> Self {
        match path.depth() % crate::N {
            0 => Player(Ply::Choice(0)),
            1 => Player(Ply::Choice(1)),
            _ => unreachable!("only 2 players supported"),
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "H{:02}", self.depth() * 10 + self.raises())
    }
}
