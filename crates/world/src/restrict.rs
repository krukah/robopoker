//! Extension of [`CfrEncoder`] for world-restricted game state generation.
//!
//! Games that support subgame solving must produce a game state identical
//! to the observed state except with the external player's private
//! information resampled to belong to a target world. The framework
//! never sees rejection sampling or game-specific dealing logic.
use crate::Belief;
use crate::Secret;
use crate::World;
use rbp_mccfr::CfrEncoder;

/// Restricts a game state so that the external player's secret belongs
/// to a target world.
///
/// Given the observed game state at the subgame entry, the implementor
/// returns a copy where public state (board, pot, stacks) is preserved
/// and only the external player's private information is resampled to
/// belong to the specified world according to the belief partition.
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
