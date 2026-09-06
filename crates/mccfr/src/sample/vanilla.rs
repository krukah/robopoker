//! Uniform sampling strategy (vanilla CFR).

use super::*;

/// Explores every branch, expanding the full tree.
///
/// **Does not work with the current solver**, whose CFR math is
/// external-sampling and assumes opponent actions were sampled.
#[derive(Debug, Clone, Copy, Default)]
pub struct VanillaSampling;

impl SamplingScheme for VanillaSampling {
    fn sample<T, E, G, I, P>(_: &P, _: &Node<T, E, G, I>, branches: Vec<Leaf<E, G>>) -> Vec<Leaf<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: CfrFlow<T = T, E = E, G = G, I = I>,
    {
        branches
    }
}
