use crate::gameplay::*;
use crate::mccfr::*;
use crate::*;

/// Record represents a single update to the blueprint profile.
/// Contains the final computed values after discounting and accumulation.
#[derive(Debug, Clone)]
pub struct Record {
    pub info: Info,
    pub edge: Edge,
    pub policy: Probability,
    pub regret: Utility,
}
