use super::edge::Edge;
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
        (0..16)
            .map(|i| ((path.0 >> (i * 4)) & 0xF) as u8)
            .filter(|&bits| bits != 0)
            .map(Edge::from)
            .take_while(|e| e.is_choice())
            .collect()
    }
}
impl From<Vec<Edge>> for Path {
    fn from(edges: Vec<Edge>) -> Self {
        assert!(edges.len() <= 16);
        edges
            .into_iter()
            .enumerate()
            .map(|(i, edge)| (u8::from(edge) as u64) << (i * 4))
            .fold(Self::default(), |Self(acc), bits| Self(acc | bits))
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

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", (self.0.wrapping_mul(2971215073)) % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_path_empty() {
        let edges = vec![];
        let round = Vec::<Edge>::from(Path::from(edges.clone()));
        assert_eq!(edges, round);
    }

    #[test]
    fn bijective_path_edges() {
        let edges = (0..)
            .map(|_| Edge::random())
            .filter(|e| e.is_choice())
            .take(16)
            .collect::<Vec<Edge>>();
        let round = Vec::<Edge>::from(Path::from(edges.clone()));
        assert_eq!(edges, round);
    }
}
