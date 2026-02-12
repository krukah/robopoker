use super::*;
use rbp_core::*;
use rbp_gameplay::*;

/// Memory represents all accumulated values for a single info.
/// Fetched in a single query via SQL join over edges.
/// Uses Edge::policy() and Edge::regret() defaults when not found.
#[derive(Debug, Clone)]
pub struct Memory {
    info: NlheInfo,
    data: Vec<(Edge, Probability, Utility, Utility, u32)>,
}

impl Memory {
    pub fn new(info: NlheInfo, data: Vec<(Edge, Probability, Utility, Utility, u32)>) -> Self {
        Self { info, data }
    }
    pub fn info(&self) -> &NlheInfo {
        &self.info
    }
    pub fn data(&self) -> &[(Edge, Probability, Utility, Utility, u32)] {
        &self.data
    }
    /// Lookup weight for edge, defaulting to Edge::policy().0 if not found.
    pub fn weight(&self, edge: &Edge) -> Probability {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map(|(_, w, _, _, _)| *w)
            .unwrap_or_else(|| edge.policy().0)
    }
    /// Lookup regret for edge, defaulting to Edge::regret().1 if not found.
    /// This preserves the fold bias from Edge::regret().
    pub fn regret(&self, edge: &Edge) -> Utility {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map(|(_, _, r, _, _)| *r)
            .unwrap_or_else(|| edge.regret().1)
    }
    /// Lookup EV for edge, defaulting to 0.0 if not found.
    pub fn evalue(&self, edge: &Edge) -> Utility {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map(|(_, _, _, v, _)| *v)
            .unwrap_or_default()
    }
    /// Lookup counts for edge, defaulting to 0 if not found.
    pub fn counts(&self, edge: &Edge) -> u32 {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map(|(_, _, _, _, c)| *c)
            .unwrap_or_default()
    }
    pub fn weights(&self) -> impl Iterator<Item = (Edge, Probability)> + '_ {
        self.data().iter().map(|(e, w, _, _, _)| (*e, *w))
    }
    pub fn regrets(&self) -> impl Iterator<Item = (Edge, Utility)> + '_ {
        self.data().iter().map(|(e, _, r, _, _)| (*e, *r))
    }
}
