use super::*;
use mccfr::*;

pub struct MiniEncoder;

impl CfrSpec for MiniEncoder {
    type F = MiniInfo;
}

impl Encoding for MiniEncoder {
    fn public<P>(&self, game: &GameOf<Self>, _: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        MiniInfo::from(*game)
    }

    fn secret(&self, _: &GameOf<Self>) -> SecretOf<Self> {}
}
