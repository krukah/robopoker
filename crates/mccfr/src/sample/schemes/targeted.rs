//! Targeted sampling strategy.

use super::*;
use monge::Density;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

/// Biases exploration toward actions with higher policy mass, concentrating
/// effort on the strategically important parts of the tree.
#[derive(Debug, Clone, Copy, Default)]
pub struct TargetedSampling;

impl SamplingScheme for TargetedSampling {
    fn sample<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        let n = branches.len();
        let p = node.game().turn();
        let walker = profile.walker();
        let chance = <TurnOf<P>>::chance();
        match (n, p) {
            (0, _) => branches,
            (_, p) if p == walker => branches,
            (_, p) if p == chance => Sampling::randomly(profile, node, branches),
            (_, p) if p != walker => TargetedSampling::explored(profile, node, branches),
            _ => unreachable!(),
        }
    }
}

impl TargetedSampling {
    /// Sample one branch with exploration bias toward high-policy actions.
    fn explored<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        let ref samples = profile.iterated_strategy(node.info());
        let index = WeightedIndex::new(
            branches
                .edges()
                .iter()
                .map(|edge| samples.density(edge))
                .map(|weight| weight.max(profile.curiosity())),
        )
        .expect("at least one policy > 0")
        .sample(&mut profile.rng(node));
        branches.only(index)
    }
}
