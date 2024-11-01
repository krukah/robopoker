#[derive(Debug, Default)]
pub struct Decision {
    pub policy: crate::Probability, // running average, not actually median
    pub regret: crate::Utility,     // cumulative non negative regret
}
