use super::edge::Edge;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);
impl Path {
    pub fn random() -> Self {
        use rand::Rng;
        Self::from(rand::thread_rng().gen::<u64>())
    }
}
impl From<Vec<Edge>> for Path {
    fn from(edges: Vec<Edge>) -> Self {
        let bits = edges
            .into_iter()
            .enumerate()
            .map(|(i, edge)| (usize::from(edge) as u64 + 1) << (i * 4))
            .fold(0u64, |acc, x| acc | x);
        Self(bits)
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

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{:02x}", self.0 % 256)
    }
}
