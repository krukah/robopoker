use super::edge::Edge;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

impl Path {
    pub fn random() -> Self {
        use rand::Rng;
        Self::from(rand::thread_rng().gen::<u64>())
    }
}

impl From<Edge> for Path {
    fn from(edge: Edge) -> Self {
        // our u8 is really u4, so we can compact 16 consecutive edges in a Path(u64) sequence
        Self(u8::from(edge) as u64)
    }
}
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
        let path = edges
            .into_iter()
            .enumerate()
            // our u8 is really u4, so we can compact 16 consecutive edges in a Path(u64) sequence
            .map(|(i, edge)| (u8::from(edge) as u64) << (i * 4))
            .fold(Self::default(), |Self(acc), bits| Self(acc | bits));
        path
    }
}

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
