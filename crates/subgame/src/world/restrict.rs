//! Extension of [`CfrEncoder`] for world-restricted game state generation.
use super::Belief;
use super::Secret;
use super::World;
use mccfr::CfrEncoder;

/// Restricts a game state so the external player's secret belongs to a target
/// world: public state (board, pot, stacks) is preserved and only the external
/// private information is resampled per the belief partition. Rejection
/// sampling and game-specific dealing stay behind this seam.
pub trait WorldRestrict<const W: usize>: CfrEncoder {
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &Self::G,
    ) -> Self::G;
}
impl<C, const W: usize> WorldRestrict<W> for &C
where
    C: WorldRestrict<W>,
{
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &Self::G,
    ) -> Self::G {
        (*self).restrict(external, world, belief, observed)
    }
}
