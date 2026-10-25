use pokerkit::*;

/// One edge of one infoset: the advantage net's estimate, the current strategy `σᵗ`, the
/// average strategy when the strategy net has just been fit, and the equilibrium target.
#[derive(serde::Serialize)]
pub struct Lane {
    edge: String,
    regret: Utility,
    current: Probability,
    average: Option<Probability>,
    nash: Probability,
}

impl Lane {
    pub fn new(
        edge: String,
        regret: Utility,
        current: Probability,
        average: Option<Probability>,
        nash: Probability,
    ) -> Self {
        Self {
            edge,
            regret,
            current,
            average,
            nash,
        }
    }
}
