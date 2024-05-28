// use super::{
//     action::RPSEdge, info::RPSInfo, node::RPSNode, player::RPSPlayer, policy::RPSPolicy,
//     strategy::RPSStrategy,
// };
use crate::cfr::rps::action::RPSEdge;
use crate::cfr::rps::info::RPSInfo;
use crate::cfr::rps::node::RPSNode;
use crate::cfr::rps::player::RPSPlayer;
use crate::cfr::rps::policy::RPSPolicy;
use crate::cfr::rps::strategy::RPSStrategy;
use crate::cfr::training::learning::profile::Profile;
use crate::cfr::training::Probability;
use crate::cfr::training::Utility;
use std::collections::HashMap;

type Signature = RPSNode<'static>;

pub(crate) struct RPSRegrets(HashMap<Signature, HashMap<RPSEdge, Utility>>);
pub(crate) struct RPSProfile<'tree> {
    _regrets: HashMap<Signature, HashMap<&'tree RPSEdge, Utility>>,
    _strates: HashMap<Signature, HashMap<&'tree RPSEdge, Probability>>,
    strategy: RPSStrategy<'tree>,
}

impl<'t> RPSProfile<'t> {
    pub fn new() -> Self {
        todo!()
        // Self {
        //     strategy: RPSStrategy::new(),
        // }
    }
    pub fn update_weight(&mut self, sign: Signature, action: &RPSEdge, weight: Probability) {
        todo!("replacement interface across Profile > Strategy > Policy")
    }

    pub fn set_weight(&mut self, sign: Signature, action: &'t RPSEdge, weight: Probability) {
        self._strates
            .entry(sign)
            .or_insert_with(HashMap::new)
            .insert(action, weight);
    }
}

impl<'t> Profile for RPSProfile<'t> {
    fn strategy(&self, _: &Self::PPlayer) -> &Self::PStrategy {
        &self.strategy
    }

    type PPlayer = RPSPlayer;
    type PAction = RPSEdge;
    type PPolicy = RPSPolicy;
    type PNode = RPSNode<'t>;
    type PInfo = RPSInfo<'t>;
    type PStrategy = RPSStrategy<'t>;
}

// this could move into different file

impl RPSRegrets {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get_regret(&self, sign: &Signature, action: &RPSEdge) -> Utility {
        *self
            .0
            .get(sign)
            .expect("regret initialized for infoset")
            .get(action)
            .expect("regret initialized for actions")
    }
    pub fn update_regret(&mut self, sign: &Signature, action: &RPSEdge, regret: Utility) {
        *self
            .0
            .get_mut(sign)
            .expect("regret initialized for infoset")
            .get_mut(action)
            .expect("regret initialized for actions") += regret;
    }
    pub fn set_regret(&mut self, sign: Signature, action: RPSEdge, regret: Utility) {
        self.0
            .entry(sign)
            .or_insert_with(HashMap::new)
            .insert(action, regret);
    }
}
