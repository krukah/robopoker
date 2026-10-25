use super::*;
use mccfr::*;
use subgame::*;

#[derive(Default)]
pub struct RpsEncoder;

impl CfrSpec for RpsEncoder {
    type F = RpsTurn;
}

impl Encoding for RpsEncoder {
    fn seed(&self, _: &GameOf<Self>) -> InfoOf<Self> {
        RpsTurn::P1
    }

    fn public<P>(&self, game: &GameOf<Self>, _: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        game.turn()
    }

    fn secret(&self, _: &GameOf<Self>) -> SecretOf<Self> {}
}

impl RpsEncoder {
    pub fn baseline(&self) -> Posterior<()> {
        Posterior::default().add((), 1.0)
    }
}

impl<const D: usize> Frontier<D> for RpsEncoder {}

impl<const W: usize> WorldRestrict<W> for RpsEncoder {
    fn restrict(
        &self,
        _: TurnOf<Self>,
        _: World,
        _: &Belief<Secret<Self>, W>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
        *observed
    }
}
