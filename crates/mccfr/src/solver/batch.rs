use crate::*;

/// One training batch's counterfactual updates, kept per tree so rayon collects each tree's
/// updates in place. Iteration is batch order: trees in order, each tree's infosets in order.
#[derive(Clone)]
pub struct Batch<F>
where
    F: CfrInfo,
{
    trees: Vec<Vec<Decisions<F>>>,
}

impl<F> From<Vec<Vec<Decisions<F>>>> for Batch<F>
where
    F: CfrInfo,
{
    fn from(trees: Vec<Vec<Decisions<F>>>) -> Self {
        Self { trees }
    }
}

impl<F> IntoIterator for Batch<F>
where
    F: CfrInfo,
{
    type Item = Decisions<F>;
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Vec<Decisions<F>>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_trees().into_iter().flatten()
    }
}

impl<F> Batch<F>
where
    F: CfrInfo,
{
    fn trees(&self) -> &[Vec<Decisions<F>>] {
        &self.trees
    }

    fn into_trees(self) -> Vec<Vec<Decisions<F>>> {
        self.trees
    }

    pub fn len(&self) -> usize {
        self.trees().iter().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.trees().iter().all(Vec::is_empty)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Decisions<F>> {
        self.trees().iter().flatten()
    }

    /// Every update, infosets in order of first appearance, each infoset's run in batch order:
    /// a stable sort on one tag per update, rather than a `Vec` per infoset.
    pub fn grouped(&self) -> Vec<&Decisions<F>> {
        let mut first = FxHashMap::<F, usize>::with_capacity_and_hasher(self.len(), Default::default());
        let mut tagged = self
            .iter()
            .map(|update| {
                let next = first.len();
                (*first.entry(*update.info()).or_insert(next), update)
            })
            .collect::<Vec<_>>();
        tagged.sort_by_key(|&(tag, _)| tag);
        tagged.into_iter().map(|(_, update)| update).collect()
    }

    /// Frees the batch on a pool worker, overlapping the next batch, not on the driving thread.
    #[cfg(feature = "server")]
    pub fn release(self)
    where
        F: 'static,
    {
        rayon::spawn(move || drop(self));
    }
    #[cfg(not(feature = "server"))]
    pub fn release(self) {}
}
