//! RPS encoder: maps game states to information sets.
use super::*;
use crate::*;

impl<R, W, S, const N: usize> Encoder for RPS<R, W, S, N>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    type T = RpsTurn;
    type E = RpsEdge;
    type G = RpsGame;
    type I = RpsInfo;
    fn seed(&self, _: &Self::G) -> Self::I {
        RpsTurn::P1
    }
    fn info(
        &self,
        _: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (_, game, _): Branch<Self::E, Self::G>,
    ) -> Self::I {
        game.turn()
    }
    fn resume(&self, _: &[Self::E], game: &Self::G) -> Self::I {
        game.turn()
    }
}
