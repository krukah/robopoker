use super::*;
use crate::*;
use deuce::*;
use pokerkit::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct GetPolicy {
    pub turn: Turn,
    pub seen: Observation,
    pub past: Vec<Action>,
    /// Starting stacks, for depth-aware reconstruction. `serde(default)` keeps
    /// old clients (and analysis-mode callers) at 100BB, so the server rebuilds
    /// exactly as before unless a caller sends real stacks (e.g. the chipzen
    /// tournament bot's off-100BB spots).
    #[serde(default = "Request::default_stacks")]
    pub stacks: [Chips; N],
}

impl GetPolicy {
    pub fn turn(&self) -> Turn {
        self.turn
    }

    pub fn seen(&self) -> Observation {
        self.seen
    }

    pub fn past(&self) -> &[Action] {
        &self.past
    }

    pub fn stacks(&self) -> [Chips; N] {
        self.stacks
    }
}
impl From<&Witness> for GetPolicy {
    fn from(recall: &Witness) -> Self {
        Self {
            turn: recall.turn(),
            seen: recall.seen(),
            past: recall.actions().iter().copied().filter(Action::is_choice).collect(),
            stacks: recall.stacks(),
        }
    }
}
