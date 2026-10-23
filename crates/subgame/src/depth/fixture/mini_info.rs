use super::*;
use mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MiniInfo {
    street: usize,
    actions: usize,
}

impl CfrInfo for MiniInfo {
    type X = MiniInfo;
    type Y = ();

    fn public(&self) -> Self::X {
        *self
    }

    fn secret(&self) -> Self::Y {}

    fn join(public: MiniInfo, _: ()) -> Self {
        public
    }
}

impl CfrPublic for MiniInfo {
    type G = MiniGame;

    fn menu(&self) -> impl Iterator<Item = EdgeOf<Self>> + use<> {
        [MiniEdge::Bet, MiniEdge::Check].into_iter()
    }

    fn count(&self) -> usize {
        2
    }
}

impl From<MiniGame> for MiniInfo {
    fn from(game: MiniGame) -> Self {
        Self {
            street: game.street(),
            actions: game.actions(),
        }
    }
}
