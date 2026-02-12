//! Uniform sampling strategy (vanilla CFR).
//!
//! **Warning**: Incompatible with the current solver implementation.

use super::*;

/// Uniform sampling strategy (vanilla CFR).
///
/// Explores all branches with equal probability.
///
/// # Compatibility Warning
///
/// **This sampling scheme does not work with the current solver.**
/// The solver uses external-sampling CFR math which assumes sampled opponent actions.
#[derive(Debug, Clone, Copy, Default)]
pub struct VanillaSampling;

impl SamplingScheme for VanillaSampling {
    fn sample<T, E, G, I, P>(
        _: &P,
        _: &Node<T, E, G, I>,
        branches: Vec<Branch<E, G>>,
    ) -> Vec<Branch<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: Profile<T = T, E = E, G = G, I = I>,
    {
        branches
    }
}
