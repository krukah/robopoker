use super::*;
use kicker::*;
use pokerkit::*;

/// All accumulated values for a single info, fetched in one SQL join over
/// edges. Missing edges fall back to the `Edge::policy()` / `Edge::regret()`
/// priors rather than to zero.
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
    /// Weight for edge, defaulting to `Edge::policy().0`.
    pub fn weight(&self, edge: &Edge) -> Probability {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map_or_else(|| edge.policy().0, |(_, w, _, _, _)| *w)
    }
    /// Regret for edge, defaulting to `Edge::regret().1` — which preserves the
    /// fold bias.
    pub fn regret(&self, edge: &Edge) -> Utility {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map_or_else(|| edge.regret().1, |(_, _, r, _, _)| *r)
    }
    /// EV for edge, defaulting to 0.0.
    pub fn payoff(&self, edge: &Edge) -> Utility {
        self.data()
            .iter()
            .find(|(e, _, _, _, _)| e == edge)
            .map(|(_, _, _, v, _)| *v)
            .unwrap_or_default()
    }
    /// Visits for edge, defaulting to 0.
    pub fn visits(&self, edge: &Edge) -> u32 {
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
