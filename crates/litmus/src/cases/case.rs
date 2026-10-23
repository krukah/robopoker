use super::*;
use serde::{Deserialize, Serialize};

/// A concrete test case. Either inline in `tests` or expanded from a `Family`.
///
/// at one history) OR `hand` + `histories` (length ≥2) (sweep histories at one hand — useful
/// for SPR-bucket monotonicity).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Case {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub kind: TestKind,
    pub edge: String,

    /// Single-state kinds.
    pub hand: Option<String>,
    /// Multi-state kinds (pair_diff has 2; monotonic has N).
    pub hands: Option<Vec<String>>,

    /// History reference, dotted: "preflop.bb_defends_2bb".
    #[serde(default)]
    pub history: String,
    /// Monotonic-over-histories: sweep N histories at a single hand.
    /// Mutually exclusive with `hands`.
    #[serde(default)]
    pub histories: Option<Vec<String>>,

    pub expect: Option<Expect>,
    #[allow(dead_code)]
    pub desired: Option<String>,
    pub diagnosis_if_violated: Option<String>,
    pub historical: Option<Historical>,
}

impl Case {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn kind(&self) -> TestKind {
        self.kind
    }

    pub fn edge(&self) -> &str {
        &self.edge
    }

    pub fn hand(&self) -> Option<&str> {
        self.hand.as_deref()
    }

    pub fn hands(&self) -> Option<&Vec<String>> {
        self.hands.as_ref()
    }

    pub fn history(&self) -> &str {
        &self.history
    }

    pub fn histories(&self) -> Option<&Vec<String>> {
        self.histories.as_ref()
    }

    pub fn expect(&self) -> Option<&Expect> {
        self.expect.as_ref()
    }

    pub fn desired(&self) -> Option<&String> {
        self.desired.as_ref()
    }

    pub fn diagnosis_if_violated(&self) -> Option<&String> {
        self.diagnosis_if_violated.as_ref()
    }

    pub fn historical(&self) -> Option<&Historical> {
        self.historical.as_ref()
    }
}
