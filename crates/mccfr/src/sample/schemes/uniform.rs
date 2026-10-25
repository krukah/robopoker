use super::*;

/// Helpers shared across the sampling module.
pub(crate) struct Sampling;

impl Sampling {
    /// Uniformly sample one branch from available choices.
    pub fn randomly<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        use rand::Rng;
        debug_assert!(!branches.is_empty());
        let n = branches.len();
        branches.only(profile.rng(node).random_range(0..n))
    }
}
