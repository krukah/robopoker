use super::*;
use crate::mccfr::*;
use crate::*;

/// A tree bundled with its precomputed cache and profile reference.
/// Owns tree, cache, and borrows profile for computing counterfactuals.
pub struct CachedTree<'a, T, E, G, I, P>
where
    T: TreeTurn,
    E: TreeEdge,
    G: TreeGame<E = E, T = T>,
    I: TreeInfo<E = E, T = T>,
    P: Profile<T = T, E = E, G = G, I = I>,
{
    tree: Tree<T, E, G, I>,
    cache: TreeCache,
    profile: &'a P,
}

impl<'a, T, E, G, I, P> CachedTree<'a, T, E, G, I, P>
where
    T: TreeTurn,
    E: TreeEdge,
    G: TreeGame<E = E, T = T>,
    I: TreeInfo<E = E, T = T>,
    P: Profile<T = T, E = E, G = G, I = I>,
{
    /// Build a CachedTree from a tree and profile.
    /// Performs two O(N) passes: top-down for reach products, bottom-up for subtree sums.
    pub fn new(tree: Tree<T, E, G, I>, profile: &'a P) -> Self {
        let mut cache = TreeCache::new(tree.n());
        cache.fill_reaches(&tree, profile);
        cache.fill_values(&tree, profile);
        Self {
            tree,
            cache,
            profile,
        }
    }
    /// Number of nodes in the tree.
    pub fn n(&self) -> usize {
        self.tree.n()
    }
    /// Compute counterfactual updates for all infosets belonging to the walker.
    /// Uses internal cache for O(1) expected_value and O(H) cfactual_value lookups.
    pub fn counterfactuals(self) -> Vec<Counterfactual<E, I>> {
        let Self {
            tree,
            cache,
            profile,
        } = self;
        let walker = profile.walker();
        tree.partition()
            .into_iter()
            .filter(|(_, infoset)| infoset.head().game().turn() == walker)
            .map(|(_, infoset)| {
                (
                    infoset.info(),
                    Self::regret_vector(&cache, &infoset, profile),
                    profile.policy_vector(&infoset),
                )
            })
            .collect()
    }
    /// Compute regret vector for an infoset using cached values.
    fn regret_vector(cache: &TreeCache, infoset: &InfoSet<T, E, G, I>, profile: &P) -> Policy<E> {
        let ref span = infoset.span();
        let ref expected = span
            .iter()
            .map(|r| cache.value(r.index()))
            .collect::<Vec<_>>();
        infoset
            .info()
            .choices()
            .into_iter()
            .map(|edge| {
                let gain = span
                    .iter()
                    .zip(expected.iter())
                    .map(|(root, &ev)| Self::cfactual_value(cache, root, &edge, profile) - ev)
                    .inspect(|r| assert!(!r.is_nan()))
                    .inspect(|r| assert!(!r.is_infinite()))
                    .sum::<Utility>();
                (edge, gain)
            })
            .collect()
    }
    /// Compute counterfactual value using cached subtree value.
    fn cfactual_value(
        cache: &TreeCache,
        root: &Node<T, E, G, I>,
        edge: &E,
        profile: &P,
    ) -> Utility {
        let child = root
            .follow(edge)
            .expect("edge belongs to outgoing branches");
        cache.value(child.index()) * profile.cfactual_reach(root)
            / cache.reach(root.index()).max(POLICY_MIN)
    }
}
