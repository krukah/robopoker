use super::*;
use atlas::*;
use regret::*;

#[derive(Default)]
pub struct RpsEncoder;

impl CfrEncoder for RpsEncoder {
    type T = RpsTurn;
    type E = RpsEdge;
    type G = RpsGame;
    type I = RpsTurn;
    const CHECK_RECALL: bool = true;

    fn seed(&self, _: &Self::G) -> Self::I {
        RpsTurn::P1
    }

    fn resume<P>(&self, _: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        game.turn()
    }
}

impl EmbeddedHistory for RpsEncoder {}
impl PublicGame for RpsEncoder {}

impl RpsEncoder {
    pub fn baseline(&self) -> Posterior<()> {
        Posterior::default().add((), 1.0)
    }
}

impl<const W: usize> WorldRestrict<W> for RpsEncoder {
    fn restrict(&self, _: Self::T, _: World, _: &Belief<Secret<Self>, W>, observed: &Self::G) -> Self::G {
        *observed
    }
}
