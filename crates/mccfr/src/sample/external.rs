//! External sampling strategy for MCCFR.

use super::*;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rbp_core::EPSILON;
use rbp_transport::Density;

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
        branches: Vec<Leaf<E, G>>,
    ) -> Vec<Leaf<E, G>>
    where
        T: CfrTurn,
        E: CfrEdge,
        G: CfrGame<E = E, T = T>,
        I: CfrInfo<E = E, T = T>,
        P: CfrFlow<T = T, E = E, G = G, I = I>,
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
    branches: Vec<Leaf<E, G>>,
) -> Vec<Leaf<E, G>>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
    P: CfrFlow<T = T, E = E, G = G, I = I>,
{
    let info = node.info();
    let rng = &mut profile.rng(node);
    let samples = &profile.sampling_distribution(info);
    let mut choices = branches;
    let weights = choices
        .iter()
        .map(|(edge, _, _)| samples.density(edge))
        .map(|weight| weight.max(EPSILON))
        .collect::<Vec<_>>();
    vec![
        choices.remove(
            WeightedIndex::new(weights)
                .expect("at least one policy > 0")
                .sample(rng),
        ),
    ]
}
