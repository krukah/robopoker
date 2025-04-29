use crate::cfr::nlhe::edge::Edge;
use crate::Arbitrary;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

impl Arbitrary for Path {
    fn random() -> Self {
        use rand::Rng;
        Self::from(rand::thread_rng().gen::<u64>())
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

impl std::iter::FromIterator<Edge> for Path {
    fn from_iter<T: IntoIterator<Item = Edge>>(iter: T) -> Self {
        let edges = iter.into_iter().collect::<Vec<_>>();
        assert!(
            edges.len() <= crate::MAX_DEPTH_SUBGAME,
            "max depth exceeded: {} {:?}",
            crate::MAX_DEPTH_SUBGAME,
            edges,
        );
        edges
            .into_iter()
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
}
