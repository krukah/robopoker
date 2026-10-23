use crate::*;
use pokerkit::*;

/// One infoset at one epoch: who acts there, and one [`Lane`] per legal edge.
#[derive(serde::Serialize)]
pub struct Reading {
    label: String,
    player: String,
    tolerance: Probability,
    lanes: Vec<Lane>,
}

impl Reading {
    pub fn new(label: String, player: String, tolerance: Probability, lanes: Vec<Lane>) -> Self {
        Self {
            label,
            player,
            tolerance,
            lanes,
        }
    }
}
