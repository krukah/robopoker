use super::*;
use kicker::*;
use pokerkit::*;

/// A single update to the blueprint profile — final values, post-discounting
/// and accumulation.
#[derive(Debug, Clone)]
pub struct Record {
    pub info: NlheInfo,
    pub edge: Edge,
    pub weight: Probability,
    pub regret: Utility,
    pub payoff: Utility,
    pub visits: u32,
}
