//! Subgame sampling for depth-limited tree building.

use super::*;

/// Subgame sampling for depth-limited solving.
///
/// Like [`ExternalSampling`], but stops expansion at chance nodes (street
/// boundaries). Frontier nodes become leaves evaluated via blueprint EV.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubgameSampling;

impl SamplingScheme for SubgameSampling {
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
        if node.game().turn().is_chance() {
            vec![]
        } else {
            ExternalSampling::sample(profile, node, branches)
        }
    }
}
