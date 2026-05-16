//! Immutable descent context fixed at subgame-solver construction time.
use crate::CfrEdge;
use crate::Descent;

/// An immutable action sequence captured at subgame-solver construction time.
///
/// Shape-identical to [`Replay`](crate::Replay) but semantically marks
/// "context that predates the solver." A [`Prefix`] must not grow after
/// construction — anything the solver appends during its own traversal
/// belongs in the solver's internal path, not here.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix<T, E>(Vec<Descent<T, E>>)
where
    T: Copy,
    E: CfrEdge;

impl<T, E> Default for Prefix<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T, E> Prefix<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    pub fn new(descents: Vec<Descent<T, E>>) -> Self {
        Self(descents)
    }

    pub fn as_slice(&self) -> &[Descent<T, E>] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<Descent<T, E>> {
        self.0
    }
}

impl<T, E> IntoIterator for Prefix<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    type Item = Descent<T, E>;
    type IntoIter = std::vec::IntoIter<Descent<T, E>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, E> IntoIterator for &'a Prefix<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    type Item = Descent<T, E>;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, Descent<T, E>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<T, E> FromIterator<Descent<T, E>> for Prefix<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Descent<T, E>>,
    {
        Self(iter.into_iter().collect())
    }
}
