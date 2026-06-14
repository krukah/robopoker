//! Full play-through descent stream: game root to current state.
use crate::CfrEdge;
use crate::Descent;

/// The full play-through from game root to the current state.
///
/// Includes chance-node transitions between streets. Use
/// [`DescentStream::current_street`](crate::DescentStream::current_street)
/// to project down to the current betting round.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Replay<T, E>(Vec<Descent<T, E>>)
where
    T: Copy,
    E: CfrEdge;

impl<T, E> Default for Replay<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T, E> Replay<T, E>
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

impl<T, E> IntoIterator for Replay<T, E>
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

impl<'a, T, E> IntoIterator for &'a Replay<T, E>
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

impl<T, E> FromIterator<Descent<T, E>> for Replay<T, E>
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
