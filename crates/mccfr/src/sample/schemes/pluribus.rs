//! Pluribus-style probabilistic pruning with warm-up: the flagship sampler.

use super::*;
use rand::Rng;

/// Regret-based pruning with the Pluribus paper's refinements, layered over
/// [`ExternalSampling`] at opponent nodes.
///
/// regrets are noisy, and pruning discards actions before their value is known.
///
/// whose value shifts as opponents evolve must not be ignored permanently.
///
/// recovered in a later phase, so correctness beats pruning efficiency there.
#[derive(Debug, Clone, Copy, Default)]
pub struct PluribusSampling;

impl SamplingScheme for PluribusSampling {
    fn sample<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        let info = node.info();
        let hyper = PruningHyperParams::get();
        if branches.is_empty() {
            return branches;
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
        let pruned = branches.clone().retain(|all, edge| {
            profile.cum_regret(info, &edge) > hyper.threshold() || all.leaf(edge).game().turn().is_terminal()
        });
        if pruned.is_empty() { branches } else { pruned }
    }
}
