//! External sampling strategy for MCCFR.

use super::*;
use monge::Density;
use pokerkit::EPSILON;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

/// Explores every action at the traverser's decision nodes, but samples a single
/// action at opponent and chance nodes according to the strategy.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalSampling;

impl SamplingScheme for ExternalSampling {
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
            (_, p) if p != walker => ExternalSampling::weighted(profile, node, branches),
            _ => unreachable!(),
        }
    }
}

impl ExternalSampling {
    /// Sample one branch weighted by sampling distribution.
    fn weighted<P>(profile: &P, node: &Node<P::F>, branches: Branches<GameOf<P>>) -> Branches<GameOf<P>>
    where
        P: Flow,
    {
        let ref samples = profile.sampling_strategy(node.info());
        let index = WeightedIndex::new(
            branches
                .edges()
                .iter()
                .map(|edge| samples.density(edge))
                .map(|weight| weight.max(EPSILON)),
        )
        .expect("at least one policy > 0")
        .sample(&mut profile.rng(node));
        branches.only(index)
    }
}
