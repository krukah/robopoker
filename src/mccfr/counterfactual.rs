use super::info::InfoSet;
use super::policy::Policy;
use super::regret::Regret;

pub struct Counterfactual {
    info: InfoSet,
    regret: Regret,
    policy: Policy,
}

impl Counterfactual {
    pub fn info(&self) -> &InfoSet {
        &self.info
    }
    pub fn regret(&self) -> &Regret {
        &self.regret
    }
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

impl From<(InfoSet, Regret, Policy)> for Counterfactual {
    fn from((info, regret, policy): (InfoSet, Regret, Policy)) -> Self {
        Self {
            info,
            regret,
            policy,
        }
    }
}
