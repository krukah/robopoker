use crate::*;
use rbp_core::*;

/// A compact sequence of abstract edges packed into 64 bits.
///
/// `Path` encodes up to 12 edges in a single `u64`, using 5 bits per edge.
/// This enables efficient storage and comparison of action sequences without
/// heap allocation.
///
/// # Encoding
///
/// Each edge maps to a 5-bit value (values 1–31, with 0 reserved for empty).
/// Edges are stored least-significant first, so the first action occupies
/// bits 0–4.
///
/// # Use Cases
///
/// - Information set keys (abstraction + path = unique info state)
/// - Strategy table lookups
/// - Subgame depth tracking (counting trailing raises)
#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

impl Path {
    const SEPARATOR: &'static str = "/";
    /// Number of edges in this path.
    pub fn length(&self) -> usize {
        (68 - self.0.leading_zeros() as usize) / 5
    }
    /// Aggression: count of trailing aggressive edges (for bet sizing grid selection).
    /// kinda wanna deprecate, dangerous if truncated
    pub fn aggression(&self) -> usize {
        self.into_iter()
            .rev()
            .take_while(Edge::is_choice)
            .filter(Edge::is_aggro)
            .count()
    }
    /// Street derived from counting Draw edges.
    /// 0 draws = Pref, 1 = Flop, 2 = Turn, 3+ = River.
    pub fn street(&self) -> rbp_cards::Street {
        match self.into_iter().filter(Edge::is_chance).count() {
            0 => rbp_cards::Street::Pref,
            1 => rbp_cards::Street::Flop,
            2 => rbp_cards::Street::Turn,
            _ => rbp_cards::Street::Rive,
        }
    }
}

impl serde::Serialize for Path {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for Path {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(d)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl Arbitrary for Path {
    fn random() -> Self {
        let n = rand::random_range(1..=rbp_core::MAX_PATH_EDGES);
        (0..n).map(|_| Edge::random()).collect()
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

impl TryFrom<&str> for Path {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        s.split(Self::SEPARATOR)
            .map(Edge::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Self::from)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            (*self)
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(Self::SEPARATOR)
        )
    }
}

impl Iterator for Path {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let x = (self.0 & 0x1F) as u8;
        if self.0 == 0 || x == 0 {
            None
        } else {
            self.0 >>= 5;
            Some(Edge::from(x))
        }
    }
}

impl DoubleEndedIterator for Path {
    fn next_back(&mut self) -> Option<Self::Item> {
        let shift = ((63u32.saturating_sub(self.0.leading_zeros())) / 5) * 5;
        let bloop = (self.0 >> shift) & 0x1F;
        if self.0 == 0 || bloop == 0 {
            None
        } else {
            self.0 &= !(0x1F << shift);
            Some(Edge::from(bloop as u8))
        }
    }
}

impl std::iter::FromIterator<Edge> for Path {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Edge>,
    {
        iter.into_iter()
            .take(rbp_core::MAX_PATH_EDGES)
            .map(u8::from)
            .map(|byte| byte as u64)
            .enumerate()
            .map(|(i, byte)| byte << (i * 5))
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
    fn bijective_str_empty() {
        let path = Path::default();
        let text = path.to_string();
        assert_eq!(text, "");
        assert_eq!(path, Path::try_from(text.as_str()).unwrap());
    }
    #[test]
    fn bijective_str_nonempty() {
        let path = (0..).map(|_| Edge::random()).take(5).collect::<Path>();
        let text = path.to_string();
        assert_eq!(path, Path::try_from(text.as_str()).unwrap());
    }

    #[test]
    fn bijective_path_edges() {
        let edges = (0..)
            .map(|_| Edge::random())
            .take(rbp_core::MAX_PATH_EDGES)
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
        let n = rand::random::<u64>() % (rbp_core::MAX_PATH_EDGES + 1) as u64;
        let n = n as usize;
        let path = (0..).map(|_| Edge::random()).take(n).collect::<Path>();
        assert_eq!(path.length(), n);
    }

    #[test]
    fn double_ended_iterator() {
        let path = (0..).map(|_| Edge::random()).take(5).collect::<Path>();
        let forward = path;
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
    fn subgame_aggression() {
        let path = [
            // this one is a late street some aggressions
            Edge::Draw,
            Edge::Raise(Odds::new(1, 2)),
            Edge::Call,
            Edge::Call,
            // new street
            Edge::Draw,
            Edge::Check,
            Edge::Check,
            Edge::Check,
            // new street
            Edge::Draw,
            Edge::Raise(Odds::new(1, 1)),
            Edge::Shove,
            Edge::Fold,
        ]
        .into_iter()
        .collect::<Path>();
        assert_eq!(path.aggression(), 2);
        let path = [
            // this one has no aggressions, new street
            Edge::Draw,
            Edge::Check,
            Edge::Check,
            Edge::Check,
        ]
        .into_iter()
        .collect::<Path>();
        assert_eq!(path.aggression(), 0);
    }
}
