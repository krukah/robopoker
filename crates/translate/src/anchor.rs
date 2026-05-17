//! Typed lattice index.

/// Opaque index into a :Translation::resolve`]:`Translation::resolve`::Lattice`]. Constructed only by the crate;
/// callers receive these from :Translation::resolve`]:`Translation::resolve`::Lattice::bracket`] or
/// :Translation::resolve`]:`Translation::resolve`::Translation::resolve`] and pattern-match.
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
