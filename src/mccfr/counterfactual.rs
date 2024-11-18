use super::info::Info;
use super::policy::Policy;
use super::regret::Regret;

pub struct Counterfactual {
    info: Info,
    regret: Regret,
    policy: Policy,
}

impl Counterfactual {
    pub fn info(&self) -> &Info {
        &self.info
    }
    pub fn regret(&self) -> &Regret {
        &self.regret
    }
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

impl From<(Info, Regret, Policy)> for Counterfactual {
    fn from((info, regret, policy): (Info, Regret, Policy)) -> Self {
        Self {
            info,
            regret,
            policy,
        }
    }
}
