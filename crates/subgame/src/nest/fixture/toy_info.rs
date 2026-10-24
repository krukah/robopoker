use super::*;
use mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToyInfo;

impl CfrInfo for ToyInfo {
    type X = ToyInfo;
    type Y = ();

    fn public(&self) -> Self::X {
        *self
    }

    fn secret(&self) -> Self::Y {}

    fn join(public: ToyInfo, (): ()) -> Self {
        public
    }
}

impl CfrPublic for ToyInfo {
    type G = ToyGame;

    fn menu(&self) -> impl Iterator<Item = EdgeOf<Self>> + use<> {
        [ToyEdge::Check, ToyEdge::Bet].into_iter()
    }

    fn count(&self) -> usize {
        2
    }
}
