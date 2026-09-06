//! Pluribus-style probabilistic pruning with warm-up: the flagship sampler.

use super::*;
use rand::Rng;

/// Regret-based pruning with the Pluribus paper's refinements, layered over
/// [`ExternalSampling`] at opponent nodes.
///
/// Each guard exists for a reason:
/// - **Warm-up** ([`PruningHyperParams::warmup`] epochs, no pruning): early
///   regrets are noisy, and pruning discards actions before their value is known.
/// - **Probabilistic exploration** ([`PruningHyperParams::explore`]): actions
///   whose value shifts as opponents evolve must not be ignored permanently.
/// - **Pre-terminal exception**: regret on a terminal-bound action can never be
///   recovered in a later phase, so correctness beats pruning efficiency there.
/// - **Threshold** ([`PruningHyperParams::threshold`]): remaining actions are
///   negative enough that equilibrium is unlikely to play them.
///
/// Unlike [`PrunableSampling`] (deterministic, no warm-up, no exploration), this
/// is the production sampler.
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
            .copied()
            .collect::<Vec<_>>();
        if pruned.is_empty() { branches } else { pruned }
    }
}
