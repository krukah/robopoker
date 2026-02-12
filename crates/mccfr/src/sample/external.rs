//! External sampling strategy for MCCFR.

use super::*;
use rbp_core::POLICY_MIN;
use rbp_transport::Density;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

/// External sampling strategy.
///
/// - Fully explores all actions at the traverser's decision nodes
/// - Samples a single action at opponent/chance nodes according to the strategy
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalSampling;

impl SamplingScheme for ExternalSampling {
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
        let n = branches.len();
        let p = node.game().turn();
        let walker = profile.walker();
        let chance = T::chance();
        match (n, p) {
            (0, _) => branches,
            (_, p) if p == walker => branches,
            (_, p) if p == chance => randomly(profile, node, branches),
            (_, p) if p != walker => weighted(profile, node, branches),
            _ => unreachable!(),
        }
    }
}

/// Sample one branch weighted by sampling distribution.
fn weighted<T, E, G, I, P>(
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
    let ref mut rng = profile.rng(info);
    let ref samples = profile.sampling_distribution(info);
    let mut choices = branches;
    let weights = choices
        .iter()
        .map(|(edge, _, _)| samples.density(edge))
        .map(|weight| weight.max(POLICY_MIN))
        .collect::<Vec<_>>();
    vec![
        choices.remove(
            WeightedIndex::new(weights)
                .expect("at least one policy > 0")
                .sample(rng),
        ),
    ]
}
