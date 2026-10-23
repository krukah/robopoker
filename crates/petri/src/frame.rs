use crate::*;
use pokerkit::*;

/// What the page draws after a step: the epoch, and one [`Reading`] per infoset it tracks.
/// `exploitability` is set only at a checkpoint, the one time the strategy net is fit.
#[derive(serde::Serialize)]
pub struct Frame {
    epoch: usize,
    t: usize,
    exploitability: Option<Utility>,
    readings: Vec<Reading>,
}

impl Frame {
    pub fn new(epoch: usize, t: usize, exploitability: Option<Utility>, readings: Vec<Reading>) -> Self {
        Self {
            epoch,
            t,
            exploitability,
            readings,
        }
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("a frame serializes")
    }
}
