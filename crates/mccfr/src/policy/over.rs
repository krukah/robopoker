//! Per-edge scalar vectors, tagged by meaning.
use crate::*;
use pokerkit::Probability;

/// A vector of per-edge scalars, tagged by what the numbers *mean*.
///
/// Regret increments and strategy distributions are both `Vec<(E, f32)>`, and both used to be
/// spelled `Policy<E>` — so passing one where the other was expected type-checked silently. The
/// [`Meaning`] marker makes it a compile error.
#[derive(Debug, Clone, PartialEq)]
pub struct Over<E, K>(Vec<(E, Probability)>, std::marker::PhantomData<K>)
where
    E: CfrEdge,
    K: Meaning;

impl<E, K> Default for Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<E, K> Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    pub fn new(scalars: Vec<(E, Probability)>) -> Self {
        Self(scalars, std::marker::PhantomData)
    }

    pub fn into_inner(self) -> Vec<(E, Probability)> {
        self.0
    }
}

impl<E, K> std::ops::Deref for Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    type Target = [(E, Probability)];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E, K> IntoIterator for Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    type Item = (E, Probability);
    type IntoIter = std::vec::IntoIter<(E, Probability)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'vector, E, K> IntoIterator for &'vector Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    type Item = &'vector (E, Probability);
    type IntoIter = std::slice::Iter<'vector, (E, Probability)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<E, K> FromIterator<(E, Probability)> for Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (E, Probability)>,
    {
        Self::new(iter.into_iter().collect())
    }
}

impl<E, K> monge::Density for Over<E, K>
where
    E: CfrEdge,
    K: Meaning,
{
    type Support = E;

    fn density(&self, x: &E) -> Probability {
        self.0.iter().find(|(e, _)| e == x).map_or(0.0, |(_, p)| *p)
    }

    fn support(&self) -> impl Iterator<Item = E> {
        self.0.iter().map(|(e, _)| *e)
    }
}
