use crate::Arbitrary;
use crate::Probability;
use crate::Utility;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Memory {
    regret: Utility,
    policy: Probability,
}

impl Memory {
    pub fn regret(&self) -> Utility {
        self.regret
    }
    pub fn policy(&self) -> Probability {
        self.policy
    }
    pub fn set_regret(&mut self, value: Utility) {
        self.regret = value;
    }
    pub fn set_policy(&mut self, value: Probability) {
        self.policy = value;
    }
    pub fn add_regret(&mut self, discount: f32, value: Utility) {
        self.regret *= discount;
        self.regret += value;
    }
    pub fn add_policy(&mut self, discount: f32, value: Probability) {
        self.policy *= discount;
        self.policy += value;
    }
}

impl From<(f32, f32)> for Memory {
    fn from((regret, policy): (f32, f32)) -> Self {
        Self { regret, policy }
    }
}

impl Arbitrary for Memory {
    fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            regret: rng.gen(),
            policy: rng.gen(),
        }
    }
}
