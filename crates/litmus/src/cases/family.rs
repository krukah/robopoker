use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A family is a template + a matrix. Cartesian-expand to N concrete cases.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Family {
    pub name_template: String,
    pub category: String,
    #[serde(default)]
    pub kind: TestKind,
    pub edge: String,

    /// Concrete fields. Any field listed in `matrix` is filled per-instance.
    pub hand: Option<String>,
    pub hands: Option<Vec<String>>,
    /// Used by monotonic kind: a sequence of hand refs that gets promoted
    /// into `hands` on each instance (no per-instance variation needed).
    pub hands_seq: Option<Vec<String>>,
    /// Used by monotonic kind: a sequence of history refs swept at the
    /// family's single `hand`. Mutually exclusive with `hands_seq`.
    pub histories_seq: Option<Vec<String>>,

    pub history: Option<String>,
    pub expect: Option<Expect>,
    pub desired: Option<String>,
    pub diagnosis_if_violated: Option<String>,
    pub historical: Option<Historical>,

    /// Cartesian product expanded over these axes. Known keys:
    /// - `hand`: `Vec<String>`
    /// - `history`: `Vec<String>`
    /// - `edge`: `Vec<String>`
    /// - `hand_pair`: Vec<[String; 2]>
    #[serde(default)]
    pub matrix: HashMap<String, Vec<serde_json::Value>>,
}

impl Family {
    pub fn name_template(&self) -> &str {
        &self.name_template
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

    pub fn hand(&self) -> Option<&String> {
        self.hand.as_ref()
    }

    pub fn hands(&self) -> Option<&Vec<String>> {
        self.hands.as_ref()
    }

    pub fn hands_seq(&self) -> Option<&Vec<String>> {
        self.hands_seq.as_ref()
    }

    pub fn histories_seq(&self) -> Option<&Vec<String>> {
        self.histories_seq.as_ref()
    }

    pub fn history(&self) -> Option<&String> {
        self.history.as_ref()
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

    pub fn matrix(&self) -> &HashMap<String, Vec<serde_json::Value>> {
        &self.matrix
    }
}
