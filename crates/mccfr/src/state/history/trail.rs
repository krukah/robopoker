//! A linear descent list, tagged by where it came from.
use crate::*;

/// A list of [`Descent`] pairs, tagged with its [`Provenance`].
///
/// The three aliases ([`Replay`], [`Prefix`], [`Story`]) are shape-identical — the marker is
/// the entire difference. Keeping them distinct types means a rollout's story cannot be handed
/// to something expecting solver-preceding context, and the fencepost question stays an active
/// decision.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Trail<G, K>(Vec<DescentOf<G>>, std::marker::PhantomData<K>)
where
    G: CfrGame,
    K: Provenance;

impl<G, K> Default for Trail<G, K>
where
    G: CfrGame,
    K: Provenance,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<G, K> Trail<G, K>
where
    G: CfrGame,
    K: Provenance,
{
    pub fn new(descents: Vec<DescentOf<G>>) -> Self {
        Self(descents, std::marker::PhantomData)
    }

    pub fn as_slice(&self) -> &[DescentOf<G>] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<DescentOf<G>> {
        self.0
    }

    /// The edges nearest first, to chain after a path into [`Encoding::public`].
    pub fn nearest(&self) -> impl Iterator<Item = G::E> + '_ {
        self.0.iter().rev().map(|d| d.1)
    }
}

/// Growth is the one capability the marker gates.
impl<G> Trail<G, Grown>
where
    G: CfrGame,
{
    pub fn push(&mut self, descent: DescentOf<G>) {
        self.0.push(descent);
    }
}

/// A rollout starts from the solver's fixed context.
impl<G> From<&Trail<G, Fixed>> for Trail<G, Grown>
where
    G: CfrGame,
{
    fn from(prefix: &Trail<G, Fixed>) -> Self {
        Self::new(prefix.as_slice().to_vec())
    }
}

impl<G, K> std::ops::Deref for Trail<G, K>
where
    G: CfrGame,
    K: Provenance,
{
    type Target = [DescentOf<G>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G, K> IntoIterator for Trail<G, K>
where
    G: CfrGame,
    K: Provenance,
{
    type Item = DescentOf<G>;
    type IntoIter = std::vec::IntoIter<DescentOf<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'trail, G, K> IntoIterator for &'trail Trail<G, K>
where
    G: CfrGame,
    K: Provenance,
{
    type Item = DescentOf<G>;
    type IntoIter = std::iter::Copied<std::slice::Iter<'trail, DescentOf<G>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl<G, K> FromIterator<DescentOf<G>> for Trail<G, K>
where
    G: CfrGame,
    K: Provenance,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = DescentOf<G>>,
    {
        Self::new(iter.into_iter().collect())
    }
}
