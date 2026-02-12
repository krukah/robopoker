//! Pluribus-style probabilistic pruning with warm-up.
//!
//! This is the flagship sampling strategy, matching the approach used in
//! Facebook AI's Pluribus — the first AI to defeat elite humans in 6-player
//! no-limit Texas Hold'em.

use super::*;
use rand::Rng;
use rbp_core::PRUNING_EXPLORE;
use rbp_core::PRUNING_THRESHOLD;
use rbp_core::PRUNING_WARMUP;

/// Pluribus-style sampling with probabilistic pruning and warm-up.
///
/// Combines regret-based pruning with refinements from the Pluribus paper,
/// balancing computational efficiency with convergence guarantees.
///
/// # Pluribus Sampling Overview
///
/// 1. **Warm-up period**: No pruning for first [`PRUNING_WARMUP`] epochs.
///    Early regrets are noisy — pruning too soon discards potentially
///    valuable actions before their true value is known.
///
/// 2. **Probabilistic exploration**: After warm-up, with probability
///    [`PRUNING_EXPLORE`] (5%), explore all branches anyway. This prevents
///    permanently ignoring actions whose value might change as opponent
///    strategies evolve.
///
/// 3. **Regret-based pruning**: Otherwise, skip actions with regret below
///    [`PRUNING_THRESHOLD`]. These actions have accumulated enough negative
///    regret that they're unlikely to be played in equilibrium.
///
/// # Training Phases
///
/// ```text
/// Epoch:     0        WARMUP                              ∞
///            |---------|--------------------------------->
///            | Warm-up |  Probabilistic Pruning          |
///            | (no     |  (95% prune, 5% explore)        |
///            | pruning)|                                 |
/// ```
///
/// # Configuration
///
/// | Constant | Value | Purpose |
/// |----------|-------|---------|
/// | `PRUNING_WARMUP` | 1k | Epochs before pruning begins |
/// | `PRUNING_EXPLORE` | 0.05 | Probability of exploring anyway |
/// | `PRUNING_THRESHOLD` | -3e5 | Regret level below which actions are pruned |
/// | `REGRET_MIN` | -1e6 | Floor for regret accumulation (allows recovery) |
///
/// # Comparison with [`PrunableSampling`]
///
/// | Aspect | PrunableSampling | PluribusSampling |
/// |--------|------------------|------------------|
/// | Warm-up | None | 1k epochs |
/// | Exploration | 0% | 5% |
/// | Determinism | Deterministic | Probabilistic |
/// | Use case | Testing, fast iteration | Production training |
///
/// # References
///
/// - Brown & Sandholm, "Superhuman AI for multiplayer poker" (Science, 2019)
/// - Brown & Sandholm, "Regret-Based Pruning in Extensive-Form Games" (NeurIPS 2015)
/// - Supplementary materials: <https://science.sciencemag.org/content/suppl/2019/07/10/science.aay2400.DC1>
#[derive(Debug, Clone, Copy, Default)]
pub struct PluribusSampling;

impl SamplingScheme for PluribusSampling {
    fn sample<T, E, G, I, P>(
        profile: &P,
        node: &Node<T, E, G, I>,
        branches: Vec<Branch<E, G>>,
    ) -> Vec<Branch<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: Profile<T = T, E = E, G = G, I = I>,
    {
        let ref info = node.info();
        if branches.is_empty() {
            return vec![];
        }
        if profile.walker() != node.game().turn() {
            return ExternalSampling::sample(profile, node, branches);
        }
        if profile.epochs() < PRUNING_WARMUP {
            return branches;
        }
        if profile.rng(info).random::<f32>() < PRUNING_EXPLORE {
            return branches;
        }
        let pruned = branches
            .iter()
            .filter(|(edge, _, _)| profile.cum_regret(info, edge) > PRUNING_THRESHOLD)
            .cloned()
            .collect::<Vec<_>>();
        if pruned.is_empty() { branches } else { pruned }
    }
}
