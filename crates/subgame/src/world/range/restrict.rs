//! Extension of [`Encoding`] for world-restricted game state generation.
use crate::Belief;
use crate::Secret;
use crate::World;
use mccfr::*;

/// Restricts a game state so the external player's secret belongs to a target
/// world: public state (board, pot, stacks) is preserved and only the external
/// private information is resampled per the belief partition. Rejection
/// sampling and game-specific dealing stay behind this seam.
pub trait WorldRestrict<const W: usize>: Encoding {
    fn restrict(
        &self,
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self>;
}

/// A tabular solver restricts worlds the way its encoder does.
impl<N, R, W, S, const B: usize, const WORLDS: usize> WorldRestrict<WORLDS> for Tabular<N, R, W, S, B>
where
    N: WorldRestrict<WORLDS>,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn restrict(
        &self,
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, WORLDS>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
        self.encoder().restrict(external, world, belief, observed)
    }
}
