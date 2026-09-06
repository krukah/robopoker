//! Deterministic regret-based pruning (RBP), without Pluribus' warm-up or
//! exploration.

use super::*;

/// Skips walker branches whose regret sits below
/// [`PruningHyperParams::threshold`] — deeply negative actions are unlikely to be
/// played, so dropping them shrinks the tree without moving the equilibrium.
/// Opponent and chance nodes delegate to [`ExternalSampling`].
///
/// If every branch would be pruned, all are kept as a safety fallback.
///
/// Faster to iterate with than [`PluribusSampling`] once regrets have
/// stabilized, but risks pruning too early; production training uses Pluribus.
///
/// Brown & Sandholm, "Regret-Based Pruning in Extensive-Form Games" (NeurIPS 2015)
#[derive(Debug, Clone, Copy, Default)]
pub struct PrunableSampling;

impl SamplingScheme for PrunableSampling {
    fn sample<T, E, G, I, P>(profile: &P, node: &Node<T, E, G, I>, branches: Vec<Leaf<E, G>>) -> Vec<Leaf<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: CfrFlow<T = T, E = E, G = G, I = I>,
    {
        let info = node.info();
        let threshold = PruningHyperParams::get().threshold();
        if branches.is_empty() {
            return vec![];
        }
        if node.game().turn() != profile.walker() {
            return ExternalSampling::sample(profile, node, branches);
        }
        let pruned = branches
            .iter()
            .filter(|(edge, _, _)| profile.cum_regret(info, edge) > threshold)
            .copied()
            .collect::<Vec<_>>();
        if pruned.is_empty() { branches } else { pruned }
    }
}
