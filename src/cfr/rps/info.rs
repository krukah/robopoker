use crate::cfr::rps::action::RPSEdge;
use crate::cfr::rps::node::RPSNode;
use crate::cfr::rps::player::RPSPlayer;
use crate::cfr::rps::signal::RPSSignal;
use crate::cfr::training::tree::info::Info;
use std::hash::{Hash, Hasher};

/// Indistinguishable states belonging to same InfoSets. Effectively, distribution of possile opponent actions.
#[derive(PartialEq, Eq)]
pub(crate) struct RPSInfo<'t> {
    roots: Vec<&'t RPSNode<'t>>,
}

impl Hash for RPSInfo<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state)
    }
}
impl<'t> Info for RPSInfo<'t> {
    type IPlayer = RPSPlayer;
    type IAction = RPSEdge;
    type ISignal = RPSSignal;
    type INode = RPSNode<'t>;
    fn roots(&self) -> &Vec<&Self::INode> {
        &self.roots
    }
    fn signal(&self) -> &Self::ISignal {
        todo!("signal")
    }
}
