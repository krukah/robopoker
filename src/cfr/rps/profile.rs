use super::bucket::RpsBucket;
use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::training::profile::Profile;
use crate::cfr::traits::Probability;
use std::collections::HashMap;

impl Profile for HashMap<RpsBucket, HashMap<RpsAction, Probability>> {
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self
    }
    type PAction = RpsAction;
    type PPlayer = RpsPlayer;
    type PPolicy = HashMap<RpsAction, Probability>;
    type PStrategy = HashMap<RpsBucket, HashMap<RpsAction, Probability>>;
    type PNode = RpsNode;
    type PInfo = RpsInfo<'static>;
}
