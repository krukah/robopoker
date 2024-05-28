use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::signal::RpsSignal;
use crate::cfr::traits::tree::info::Info;
use std::hash::{Hash, Hasher};

/// Indistinguishable states belonging to same InfoSets. Effectively, distribution of possile opponent actions.
#[derive(PartialEq, Eq)]
pub(crate) struct RpsInfo<'t> {
    roots: Vec<&'t RpsNode<'t>>,
}

impl Hash for RpsInfo<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state)
    }
}
impl<'t> Info for RpsInfo<'t> {
    type IPlayer = RpsPlayer;
    type IAction = RpsAction;
    type ISignal = RpsSignal;
    type INode = RpsNode<'t>;
    fn roots(&self) -> &Vec<&Self::INode> {
        &self.roots
    }
    fn signal(&self) -> Self::ISignal {
        RpsSignal {}
    }
}
