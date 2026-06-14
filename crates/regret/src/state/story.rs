//! Growing descent story emitted during a biased rollout at a subgame frontier.
use crate::CfrEdge;
use crate::Descent;
use crate::Prefix;

/// An edge sequence accumulated during a single biased rollout at a subgame
/// frontier.
///
/// Starts from a base (typically a [`Prefix`] or a solver-reconstructed
/// path-to-frontier) and grows as the rollout samples chance outcomes and
/// biased actions. Distinct from both [`Replay`](crate::Replay) and
/// [`Prefix`] to prevent feeding a story into a call expecting
/// root-anchored context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Story<T, E>(Vec<Descent<T, E>>)
where
    T: Copy,
    E: CfrEdge;

impl<T, E> Default for Story<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T, E> Story<T, E>
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

    pub fn push(&mut self, descent: Descent<T, E>) {
        self.0.push(descent);
    }

    pub fn into_inner(self) -> Vec<Descent<T, E>> {
        self.0
    }
}

impl<T, E> IntoIterator for Story<T, E>
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

impl<'a, T, E> IntoIterator for &'a Story<T, E>
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

impl<T, E> From<&Prefix<T, E>> for Story<T, E>
where
    T: Copy,
    E: CfrEdge,
{
    fn from(prefix: &Prefix<T, E>) -> Self {
        Self(prefix.as_slice().to_vec())
    }
}
