#[derive(Debug, Default)]
pub struct Strategy {
    pub policy: crate::Probability, // most recent
    pub advice: crate::Probability, // running average, not actually median
    pub regret: crate::Utility,     // cumulative non negative regret
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, " POLICY: {:<8.3}", self.policy)?;
        write!(f, " ADVICE: {:<8.3}", self.advice)?;
        // write!(f, " REGRET: {:<8.3}", self.regret)?;
        Ok(())
    }
}
