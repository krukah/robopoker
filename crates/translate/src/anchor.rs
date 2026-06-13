//! Typed lattice index.

/// Opaque index into a [`crate::Lattice`]. Constructed only by the crate;
/// callers receive these from [`crate::Lattice::bracket`] or
/// [`crate::Translation::resolve`] and pattern-match.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Anchor(usize);

impl Anchor {
    pub const fn new(idx: usize) -> Self {
        Self(idx)
    }

    /// Index into the originating lattice.
    pub const fn idx(&self) -> usize {
        self.0
    }
}
