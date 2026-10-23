//! Stable short hash of a game tree's [`Shape`].

/// A game tree's identity, condensed. Tags scoreboard rows so a persisted score is attributable
/// to the exact tree that produced it.
///
/// `DefaultHasher`'s seeds are fixed, so the value is stable across processes and builds —
/// which is the only reason it is safe to persist. It is a `u64` because that is what the
/// hasher yields; the sixteen hex digits are how it is *written*, not what it is.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Digest(u64);

impl From<&super::Shape> for Digest {
    fn from(shape: &super::Shape) -> Self {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        shape.to_string().hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}
