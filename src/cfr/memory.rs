pub struct Memory {
    pub policy: crate::Probability, // most recent
    pub advice: crate::Probability, // running average, not actually median
    pub regret: crate::Utility,     // cumulative non negative regret
}

impl Memory {
    pub fn new() -> Self {
        Self {
            policy: 0.0,
            advice: 0.0,
            regret: 0.0,
        }
    }
}
