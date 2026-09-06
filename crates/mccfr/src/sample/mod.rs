//! Sampling strategies for MCCFR tree traversal: which branches [`TreeBuilder`]
//! explores, trading off variance against convergence speed and cost.
//!
//! All are zero-cost unit structs selected via the [`Solver::S`] associated type.
//!
//! | Strategy | Walker Nodes | Opponent Nodes | Use Case |
//! |----------|--------------|----------------|----------|
//! | [`ExternalSampling`] | Explore all | Sample one | Standard MCCFR |
//! | [`TargetedSampling`] | Explore all | Bias toward high-policy | Focused exploration |
//! | [`VanillaSampling`] | Explore all | Explore all | Full tree (expensive) |
//! | [`PrunableSampling`] | Prune low-regret | Sample one | Deterministic pruning |
//! | [`PluribusSampling`] | Prune + explore 5% | Sample one | Production (Pluribus) |
//!
//! - External sampling: Lanctot et al., "Monte Carlo Sampling for Regret Minimization"
//! - Pruning: Brown & Sandholm, "Regret-Based Pruning in Extensive-Form Games"
//! - Pluribus: Brown & Sandholm, "Superhuman AI for multiplayer poker" (Science, 2019)

mod external;
mod pluribus;
mod pruning;
mod targeted;
mod vanilla;

pub use external::*;
pub use pluribus::*;
pub use pruning::*;
pub use targeted::*;
pub use vanilla::*;

use crate::*;

/// Which branches [`TreeBuilder`] explores at each node, invoked after the
/// encoder generates candidates and before expansion continues.
///
/// Implementors: return empty only if `branches` was empty, and draw randomness
/// from `profile.rng(node)` so traversals stay deterministic per epoch.
pub trait SamplingScheme {
    /// Filter or sample branches into [`TreeBuilder`]'s expansion queue.
    fn sample<T, E, G, I, P>(profile: &P, node: &Node<T, E, G, I>, branches: Vec<Leaf<E, G>>) -> Vec<Leaf<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: CfrFlow<T = T, E = E, G = G, I = I>;
}

/// Uniformly sample one branch from available choices.
pub fn randomly<T, E, G, I, P>(profile: &P, node: &Node<T, E, G, I>, branches: Vec<Leaf<E, G>>) -> Vec<Leaf<E, G>>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    P: CfrFlow<T = T, E = E, G = G, I = I>,
{
    use rand::Rng;
    debug_assert!(!branches.is_empty());
    let n = branches.len();
    let mut choices = branches;
    let ref mut rng = profile.rng(node);
    vec![choices.remove(rng.random_range(0..n))]
}
