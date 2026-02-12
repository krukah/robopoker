//! NLHE edge type: discretized betting actions.
use rbp_core::Probability;
use rbp_core::Utility;
use rbp_gameplay::Action;
use rbp_gameplay::Edge;
use rbp_gameplay::Odds;
use rbp_mccfr::*;
use rbp_transport::Support;

/// NLHE edge type for CFR tree traversal.
///
/// Newtype wrapper around gameplay `Edge` for NLHE-specific CFR.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NlheEdge(Edge);

impl NlheEdge {
    /// True if this is a player decision (not a chance node).
    pub fn is_choice(&self) -> bool {
        self.0.is_choice()
    }
    /// True if this is a chance node (Draw).
    pub fn is_chance(&self) -> bool {
        self.0.is_chance()
    }
    /// True if this is an aggressive action (raise or shove).
    pub fn is_aggro(&self) -> bool {
        self.0.is_aggro()
    }
    /// True if this is a shove (all-in).
    pub fn is_shove(&self) -> bool {
        self.0.is_shove()
    }
    /// True if this is a raise (not all-in).
    pub fn is_raise(&self) -> bool {
        self.0.is_raise()
    }
    /// Default policy for CFR initialization.
    pub fn default_policy(&self) -> Probability {
        self.0.policy().0
    }
    /// Default regret for CFR initialization (biased warmstart).
    pub fn default_regret(&self) -> Utility {
        self.0.regret().1
    }
}

impl Support for NlheEdge {}
impl CfrEdge for NlheEdge {}

impl From<Edge> for NlheEdge {
    fn from(edge: Edge) -> Self {
        Self(edge)
    }
}
impl From<NlheEdge> for Edge {
    fn from(edge: NlheEdge) -> Self {
        edge.0
    }
}
impl AsRef<Edge> for NlheEdge {
    fn as_ref(&self) -> &Edge {
        &self.0
    }
}
impl From<Odds> for NlheEdge {
    fn from(odds: Odds) -> Self {
        Self(Edge::from(odds))
    }
}
impl From<Action> for NlheEdge {
    fn from(action: Action) -> Self {
        Self(Edge::from(action))
    }
}
impl From<u64> for NlheEdge {
    fn from(value: u64) -> Self {
        Self(Edge::from(value))
    }
}
impl From<NlheEdge> for u64 {
    fn from(edge: NlheEdge) -> Self {
        u64::from(Edge::from(edge))
    }
}

impl std::fmt::Display for NlheEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Edge::from(*self))
    }
}
