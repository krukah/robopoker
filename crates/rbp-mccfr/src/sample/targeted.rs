//! Targeted sampling strategy.

use super::*;
use rbp_transport::Density;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

/// Targeted sampling strategy.
///
/// Focuses exploration on strategically important parts of the tree
/// by biasing toward actions with higher policy mass.
#[derive(Debug, Clone, Copy, Default)]
pub struct TargetedSampling;

impl SamplingScheme for TargetedSampling {
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
            (_, p) if p != walker => explored(profile, node, branches),
            _ => unreachable!(),
        }
    }
}

/// Sample one branch with exploration bias toward high-policy actions.
fn explored<T, E, G, I, P>(
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
    let ref samples = profile.iterated_distribution(info);
    let mut choices = branches;
    let weights = choices
        .iter()
        .map(|(edge, _, _)| samples.density(edge))
        .map(|weight| weight.max(profile.curiosity()))
        .collect::<Vec<_>>();
    vec![
        choices.remove(
            WeightedIndex::new(weights)
                .expect("at least one policy > 0")
                .sample(rng),
        ),
    ]
}
