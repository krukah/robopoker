use crate::gameplay::edge::Edge;
use crate::Arbitrary;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

impl Path {
    pub fn length(&self) -> usize {
        (67 - self.0.leading_zeros() as usize) / 4
    }
    pub fn raises(&self) -> usize {
        self.into_iter()
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count()
    }
}

impl Arbitrary for Path {
    fn random() -> Self {
        Self::from(rand::random::<u64>())
    }
}

/// Vec<Edge> isomorphism
/// we (un)pack the byte representation of the edges in a Path(u64) sequence
impl From<Path> for Vec<Edge> {
    fn from(path: Path) -> Self {
        path.into_iter().collect()
    }
}

impl From<Vec<Edge>> for Path {
    fn from(edges: Vec<Edge>) -> Self {
        edges.into_iter().collect()
    }
}

/// u64 isomorphism
/// trivial unpacking and packing
impl From<u64> for Path {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<Path> for u64 {
    fn from(path: Path) -> Self {
        path.0
    }
}
impl From<Path> for i64 {
    fn from(path: Path) -> Self {
        path.0 as i64
    }
}
impl From<i64> for Path {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.clone()
            .into_iter()
            .try_for_each(|e| write!(f, ".{}", e))
    }
}

impl Iterator for Path {
    type Item = Edge;
    fn next(&mut self) -> Option<Self::Item> {
        let x = (self.0 & 0xF) as u8;
        if self.0 == 0 {
            None
        } else if x == 0 {
            None
        } else {
            self.0 >>= 4;
            Some(Edge::from(x))
        }
    }
}

impl DoubleEndedIterator for Path {
    fn next_back(&mut self) -> Option<Self::Item> {
        let shift = ((63u32.saturating_sub(self.0.leading_zeros())) / 4) * 4;
        let bloop = (self.0 >> shift) & 0xF;
        if self.0 == 0 {
            None
        } else if bloop == 0 {
            None
        } else {
            self.0 &= !(0xF << shift);
            Some(Edge::from(bloop as u8))
        }
    }
}

impl std::iter::FromIterator<Edge> for Path {
    fn from_iter<T: IntoIterator<Item = Edge>>(iter: T) -> Self {
        iter.into_iter()
            .take(crate::MAX_DEPTH_SUBGAME)
            .map(u8::from)
            .map(|byte| byte as u64)
            .enumerate()
            .map(|(i, byte)| byte << (i * 4))
            .fold(0u64, |acc, bits| acc | bits)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::odds::Odds;

    #[test]
    fn bijective_path_empty() {
        let edges = vec![];
        let paths = Vec::<Edge>::from(Path::from(edges.clone()));
        assert_eq!(edges, paths);
    }

    #[test]
    fn bijective_path_edges() {
        let edges = (0..)
            .map(|_| Edge::random())
            .take(crate::MAX_DEPTH_SUBGAME)
            .collect::<Vec<Edge>>();
        let paths = Vec::<Edge>::from(Path::from(edges.clone()));
        assert_eq!(edges, paths);
    }

    #[test]
    fn bijective_path_collect() {
        let edges = (0..).map(|_| Edge::random()).take(5).collect::<Vec<Edge>>();
        let collected = Path::from(edges.clone()).into_iter().collect::<Vec<Edge>>();
        assert_eq!(edges, collected);
    }

    #[test]
    fn length() {
        let n = rand::random::<u64>() % (crate::MAX_DEPTH_SUBGAME + 1) as u64;
        let n = n as usize;
        let path = (0..).map(|_| Edge::random()).take(n).collect::<Path>();
        assert_eq!(path.length(), n);
    }

    #[test]
    fn double_ended_iterator() {
        let path = (0..).map(|_| Edge::random()).take(5).collect::<Path>();
        let forward = path.clone();
        let reverse = path
            .into_iter()
            .rev()
            .collect::<Vec<Edge>>()
            .into_iter()
            .rev()
            .collect::<Path>();
        assert_eq!(forward, reverse);
    }

    #[test]
    fn subgame_raises() {
        let path = [
            // this one is a late street some aggressions
            Edge::Draw,
            Edge::Raise(Odds::random()),
            Edge::Call,
            Edge::Call,
            // new street
            Edge::Draw,
            Edge::Check,
            Edge::Check,
            Edge::Check,
            // new street
            Edge::Draw,
            Edge::Raise(Odds::random()),
            Edge::Shove,
            Edge::Fold,
        ]
        .into_iter()
        .collect::<Path>();
        assert_eq!(path.raises(), 2);

        let path = [
            // this one has no aggressions, new street
            Edge::Draw,
            Edge::Check,
            Edge::Check,
            Edge::Check,
        ]
        .into_iter()
        .collect::<Path>();
        assert_eq!(path.raises(), 0);
    }
}
