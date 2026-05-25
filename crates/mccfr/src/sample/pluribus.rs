//! Pluribus-style probabilistic pruning with warm-up.
//!
//! This is the flagship sampling strategy, matching the approach used in
//! Facebook AI's Pluribus — the first AI to defeat elite humans in 6-player
//! no-limit Texas Hold'em.

use super::*;
use rand::Rng;

/// Pluribus-style sampling with probabilistic pruning and warm-up.
///
/// Combines regret-based pruning with refinements from the Pluribus paper,
/// balancing computational efficiency with convergence guarantees.
///
/// # Pluribus Sampling Overview
///
/// 1. **Warm-up period**: No pruning for first [`PruningHyperParams::warmup`]
///    epochs. Early regrets are noisy — pruning too soon discards potentially
///    valuable actions before their true value is known.
///
/// 2. **Probabilistic exploration**: After warm-up, with probability
///    [`PruningHyperParams::explore`] (5%), explore all branches anyway.
///    This prevents permanently ignoring actions whose value might change
///    as opponent strategies evolve.
///
/// 3. **Pre-terminal exception**: Actions leading directly to a terminal
///    node are never pruned. Regret on those actions cannot be recovered
///    from any future phase, so getting the strategy right matters more
///    than pruning efficiency.
///
/// 4. **Regret-based pruning**: Otherwise, skip actions with regret below
///    [`PruningHyperParams::threshold`]. These actions have accumulated enough
///    negative regret that they're unlikely to be played in equilibrium.
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
/// | Field | Value | Purpose |
/// |----------|-------|---------|
/// | `PruningHyperParams::warmup` | 16k | Epochs before pruning begins |
/// | `PruningHyperParams::explore` | 0.05 | Probability of exploring anyway |
/// | `PruningHyperParams::threshold` | -3e5 | Regret level below which actions are pruned |
/// | `TrainingHyperParams::regret_min` | -4e6 | Floor for regret accumulation (allows recovery) |
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
    fn sample<T, E, G, I, P>(profile: &P, node: &Node<T, E, G, I>, branches: Vec<Leaf<E, G>>) -> Vec<Leaf<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: CfrFlow<T = T, E = E, G = G, I = I>,
    {
        let info = node.info();
        let hyper = PruningHyperParams::get();
        if branches.is_empty() {
            return vec![];
        }
        if profile.walker() != node.game().turn() {
            return ExternalSampling::sample(profile, node, branches);
        }
        if profile.t() < hyper.warmup() {
            return branches;
        }
        if profile.rng(node).random::<f32>() < hyper.explore() {
            return branches;
        }
        let pruned = branches
            .iter()
            .filter(|(edge, game, _)| game.turn().is_terminal() || profile.cum_regret(info, edge) > hyper.threshold())
            .cloned()
            .collect::<Vec<_>>();
        if pruned.is_empty() { branches } else { pruned }
    }
}
