use super::*;
use rbp_core::*;
use rbp_gameplay::*;

/// Record represents a single update to the blueprint profile.
/// Contains the final computed values after discounting and accumulation.
#[derive(Debug, Clone)]
pub struct Record {
    pub info: NlheInfo,
    pub edge: Edge,
    pub weight: Probability,
    pub regret: Utility,
    pub evalue: Utility,
    pub counts: u32,
}
