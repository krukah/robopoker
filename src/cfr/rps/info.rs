use super::{action::RPSEdge, node::RPSNode, player::RPSPlayer};
use crate::cfr::training::info::Info;
use std::hash::{Hash, Hasher};

/// Indistinguishable states belonging to same InfoSets. Effectively, distribution of possile opponent actions.
#[derive(PartialEq, Eq)]
pub(crate) struct RPSInfo<'t> {
    roots: Vec<&'t RPSNode<'t>>,
}

type Signature = RPSNode<'static>;
impl RPSInfo<'_> {
    pub fn signature(&self) -> Signature {
        todo!("owned signature returns for Info + Node")
    }
}

impl Hash for RPSInfo<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state)
    }
}
impl<'t> Info for RPSInfo<'t> {
    type IPlayer = RPSPlayer;
    type IAction = RPSEdge;
    type INode = RPSNode<'t>;
    fn roots(&self) -> &Vec<&Self::INode> {
        &self.roots
    }
}
