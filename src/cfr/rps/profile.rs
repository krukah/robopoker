use super::signal::RpsSignal;
use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::training::learning::profile::Profile;
use crate::cfr::training::Probability;
use std::collections::HashMap;

impl Profile for HashMap<RpsSignal, HashMap<RpsEdge, Probability>> {
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self
    }
    type PAction = RpsEdge;
    type PPlayer = RpsPlayer;
    type PPolicy = HashMap<RpsEdge, Probability>;
    type PStrategy = HashMap<RpsSignal, HashMap<RpsEdge, Probability>>;
    type PNode = RpsNode<'static>;
    type PInfo = RpsInfo<'static>;
}
