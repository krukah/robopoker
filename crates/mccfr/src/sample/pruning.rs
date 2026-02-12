//! Regret-based pruning (RBP) sampling strategy.
//!
//! Deterministic pruning that skips actions with deeply negative regret.
//! Simpler than [`PluribusSampling`] but lacks warm-up and exploration.

use super::*;
use rbp_core::PRUNING_THRESHOLD;

/// Deterministic regret-based pruning (RBP) sampling strategy.
///
/// Prunes branches whose regret has fallen below [`PRUNING_THRESHOLD`],
/// reducing tree size while preserving convergence. Actions that have
/// accumulated enough negative regret are unlikely to be played, so
/// skipping them saves computation without significantly affecting results.
///
/// # Algorithm
///
/// At walker decision nodes:
/// 1. Filter branches where `cum_regret(info, edge) > PRUNING_THRESHOLD`
/// 2. If all branches would be pruned, keep all (safety fallback)
/// 3. Expand only the surviving branches
///
/// At opponent/chance nodes: delegates to [`ExternalSampling`].
///
/// # Tradeoffs vs [`PluribusSampling`]
///
/// | Aspect | PrunableSampling | PluribusSampling |
/// |--------|------------------|------------------|
/// | Warm-up | None | 1000 epochs |
/// | Exploration | 0% | 5% |
/// | Determinism | Yes | No |
/// | Risk | May prune too early | Slower convergence |
///
/// Use `PrunableSampling` for faster iteration when you trust regrets have
/// stabilized. Use [`PluribusSampling`] for production training.
///
/// # References
///
/// Brown & Sandholm, "Regret-Based Pruning in Extensive-Form Games" (NeurIPS 2015)
#[derive(Debug, Clone, Copy, Default)]
pub struct PrunableSampling;

impl SamplingScheme for PrunableSampling {
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
        if node.game().turn() != profile.walker() {
            return ExternalSampling::sample(profile, node, branches);
        }
        let pruned = branches
            .iter()
            .filter(|(edge, _, _)| profile.cum_regret(info, edge) > PRUNING_THRESHOLD)
            .cloned()
            .collect::<Vec<_>>();
        if pruned.is_empty() { branches } else { pruned }
    }
}
