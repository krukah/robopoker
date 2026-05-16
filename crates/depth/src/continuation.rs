//! Opaque continuation strategy index for depth-limited frontier evaluation.

/// Index of a continuation strategy at a depth-limited frontier.
///
/// The number of continuations D is determined by the const generic
/// on the containing types. The implementor of [`DepthSampler`] decides
/// what each index means for their game.
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
