use super::*;

/// Which branches [`TreeBuilder`] explores at each node, invoked after the
/// encoder generates candidates and before expansion continues.
///
/// Implementors: return empty only if `branches` was empty, and draw randomness
/// from `profile.rng(node)` so traversals stay deterministic per epoch.
pub trait SamplingScheme {
    /// Filter or sample branches into [`TreeBuilder`]'s expansion queue.
    fn sample<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow;
}
