use super::*;
use kicker::*;
use pokerkit::*;

/// A single update to the blueprint profile — final values, post-discounting
/// and accumulation.
#[derive(Debug, Clone)]
pub struct Record {
    info: NlheInfo,
    edge: Edge,
    weight: Probability,
    regret: Utility,
    payoff: Utility,
    visits: u32,
}

impl Record {
    pub fn new(info: NlheInfo, edge: Edge, weight: Probability, regret: Utility, payoff: Utility, visits: u32) -> Self {
        Self {
            info,
            edge,
            weight,
            regret,
            payoff,
            visits,
        }
    }

    pub fn info(&self) -> &NlheInfo {
        &self.info
    }

    pub fn edge(&self) -> &Edge {
        &self.edge
    }

    pub fn weight(&self) -> Probability {
        self.weight
    }

    pub fn regret(&self) -> Utility {
        self.regret
    }

    pub fn payoff(&self) -> Utility {
        self.payoff
    }

    pub fn visits(&self) -> u32 {
        self.visits
    }
}
