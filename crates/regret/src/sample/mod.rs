//! Sampling strategies for MCCFR tree traversal.
//!
//! Controls which branches are explored during [`TreeBuilder`] construction.
//! Different strategies trade off between variance, convergence speed, and
//! computational cost.
//!
//! # Available Strategies
//!
//! | Strategy | Walker Nodes | Opponent Nodes | Use Case |
//! |----------|--------------|----------------|----------|
//! | [`ExternalSampling`] | Explore all | Sample one | Standard MCCFR |
//! | [`TargetedSampling`] | Explore all | Bias toward high-policy | Focused exploration |
//! | [`VanillaSampling`] | Explore all | Explore all | Full tree (expensive) |
//! | [`PrunableSampling`] | Prune low-regret | Sample one | Deterministic pruning |
//! | [`PluribusSampling`] | Prune + explore 5% | Sample one | Production (Pluribus) |
//!
//! # Composition
//!
//! All strategies are zero-cost unit structs selected via the [`Solver::S`] associated type.
//!
//! # References
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

/// Trait for sampling strategies in Monte Carlo CFR variants.
///
/// Implementations control which branches [`TreeBuilder`] explores at each node.
/// The strategy is invoked after the encoder generates candidate branches,
/// filtering or sampling them before tree expansion continues.
///
/// # Implementor Guidelines
///
/// - Return `branches` unchanged to explore all actions
/// - Return a subset to prune or sample
/// - Return empty vec only if `branches` was empty (terminal node)
/// - Use `profile.rng(node)` for deterministic randomness
pub trait SamplingScheme {
    /// Filter or sample branches for tree expansion.
    ///
    /// Called by [`TreeBuilder`] at each node during tree construction.
    /// The returned branches will be added to the expansion queue.
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
