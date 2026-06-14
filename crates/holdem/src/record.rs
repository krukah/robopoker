use super::*;
use cowboys::*;
use pokerkit::*;

/// Record represents a single update to the blueprint profile.
/// Contains the final computed values after discounting and accumulation.
#[derive(Debug, Clone)]
pub struct Record {
    pub info: NlheInfo,
    pub edge: Edge,
    pub weight: Probability,
    pub regret: Utility,
    pub payoff: Utility,
    pub visits: u32,
}
