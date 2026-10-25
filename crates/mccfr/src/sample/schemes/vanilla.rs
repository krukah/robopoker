//! Uniform sampling strategy (vanilla CFR).

use super::*;

/// Explores every branch, expanding the full tree.
///
/// **Does not work with the current solver**, whose CFR math is
/// external-sampling and assumes opponent actions were sampled.
#[derive(Debug, Clone, Copy, Default)]
pub struct VanillaSampling;

impl SamplingScheme for VanillaSampling {
    fn sample<P>(_: &P, _: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        branches
    }
}
