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

impl std::fmt::Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " POLICY: {:<8.3}", self.policy)?;
        write!(f, " ADVICE: {:<8.3}", self.advice)?;
        write!(f, " REGRET: {:<8.3}", self.regret)?;
        Ok(())
    }
}
