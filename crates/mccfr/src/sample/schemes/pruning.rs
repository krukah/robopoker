//! Deterministic regret-based pruning (RBP), without Pluribus' warm-up or
//! exploration.

use super::*;

/// Skips walker branches whose regret sits below [`PruningHyperParams::threshold`] — deeply
/// negative actions are unlikely to be played, so dropping them shrinks the tree without moving
/// the equilibrium. Opponent and chance nodes delegate to [`ExternalSampling`].
///
/// If every branch would be pruned, all are kept as a safety fallback.
#[derive(Debug, Clone, Copy, Default)]
pub struct PrunableSampling;

impl SamplingScheme for PrunableSampling {
    fn sample<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        let info = node.info();
        let threshold = PruningHyperParams::get().threshold();
        if branches.is_empty() {
            return branches;
        }
        if node.game().turn() != profile.walker() {
            return ExternalSampling::sample(profile, node, branches);
        }
        let pruned = branches
            .clone()
            .retain(|_, edge| profile.cum_regret(info, &edge) > threshold);
        if pruned.is_empty() { branches } else { pruned }
    }
}
