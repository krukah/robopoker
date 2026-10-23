//! Opaque continuation strategy index for depth-limited frontier evaluation.

/// Index of a continuation strategy at a depth-limited frontier. The count D
/// is a const generic on the containing types; what an index *means* is up to
/// the `DepthSampler` implementor.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Continuation(usize);

impl Continuation {
    pub fn all<const D: usize>() -> impl Iterator<Item = Self> {
        (0..D).map(Self)
    }

    pub fn index(self) -> usize {
        self.0
    }
}
