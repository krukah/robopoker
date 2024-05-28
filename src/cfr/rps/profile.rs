// use super::{
//     action::RpsEdge, info::RpsInfo, node::RpsNode, player::RpsPlayer, policy::RpsPolicy,
//     strategy::RpsStrategy,
// };
use crate::cfr::rps::action::RpsEdge;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::rps::policy::RpsPolicy;
use crate::cfr::rps::strategy::RpsStrategy;
use crate::cfr::training::learning::profile::Profile;
use crate::cfr::training::Probability;
use crate::cfr::training::Utility;
use std::collections::HashMap;

type Signature = RpsNode<'static>;

pub(crate) struct RpsRegrets(HashMap<Signature, HashMap<RpsEdge, Utility>>);
pub(crate) struct RpsProfile<'tree> {
    _regrets: HashMap<Signature, HashMap<&'tree RpsEdge, Utility>>,
    _strates: HashMap<Signature, HashMap<&'tree RpsEdge, Probability>>,
    strategy: RpsStrategy<'tree>,
}

impl<'t> RpsProfile<'t> {
    pub fn new() -> Self {
        todo!()
        // Self {
        //     strategy: RpsStrategy::new(),
        // }
    }
    pub fn update_weight(&mut self, sign: Signature, action: &RpsEdge, weight: Probability) {
        todo!("replacement interface across Profile > Strategy > Policy")
    }

    pub fn set_weight(&mut self, sign: Signature, action: &'t RpsEdge, weight: Probability) {
        self._strates
            .entry(sign)
            .or_insert_with(HashMap::new)
            .insert(action, weight);
    }
}

impl<'t> Profile for RpsProfile<'t> {
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self.strategy
    }

    type PPlayer = RpsPlayer;
    type PAction = RpsEdge;
    type PPolicy = RpsPolicy;
    type PNode = RpsNode<'t>;
    type PInfo = RpsInfo<'t>;
    type PStrategy = RpsStrategy<'t>;
}

// this could move into different file

impl RpsRegrets {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get_regret(&self, sign: &Signature, action: &RpsEdge) -> Utility {
        *self
            .0
            .get(sign)
            .expect("regret initialized for infoset")
            .get(action)
            .expect("regret initialized for actions")
    }
    pub fn update_regret(&mut self, sign: &Signature, action: &RpsEdge, regret: Utility) {
        *self
            .0
            .get_mut(sign)
            .expect("regret initialized for infoset")
            .get_mut(action)
            .expect("regret initialized for actions") += regret;
    }
    pub fn set_regret(&mut self, sign: Signature, action: RpsEdge, regret: Utility) {
        self.0
            .entry(sign)
            .or_insert_with(HashMap::new)
            .insert(action, regret);
    }
}
