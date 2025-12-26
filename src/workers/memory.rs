use crate::gameplay::*;
use crate::mccfr::*;
use crate::*;

/// Memory represents all accumulated values for a single info.
/// Fetched in a single query via SQL join over edges.
/// Uses Edge::policy() and Edge::regret() defaults when not found.
#[derive(Debug, Clone)]
pub struct Memory {
    info: Info,
    data: Vec<(Edge, Probability, Utility)>,
}

impl Memory {
    pub fn new(info: Info, data: Vec<(Edge, Probability, Utility)>) -> Self {
        Self { info, data }
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn data(&self) -> &[(Edge, Probability, Utility)] {
        &self.data
    }
    /// Lookup policy for edge, defaulting to Edge::policy().0 if not found.

    pub fn policy(&self, edge: &Edge) -> Probability {
        self.data()
            .iter()
            .find(|(e, _, _)| e == edge)
            .map(|(_, p, _)| *p)
            .unwrap_or_else(|| edge.policy().0)
    }
    /// Lookup regret for edge, defaulting to Edge::regret().1 if not found.
    /// This preserves the fold bias from Edge::regret().

    pub fn regret(&self, edge: &Edge) -> Utility {
        self.data()
            .iter()
            .find(|(e, _, _)| e == edge)
            .map(|(_, _, r)| *r)
            .unwrap_or_else(|| edge.regret().1)
    }

    pub fn policies(&self) -> impl Iterator<Item = (Edge, Probability)> + '_ {
        self.data().iter().map(|(e, p, _)| (*e, *p))
    }

    pub fn regrets(&self) -> impl Iterator<Item = (Edge, Utility)> + '_ {
        self.data().iter().map(|(e, _, r)| (*e, *r))
    }
}
