//! Snapshot of training progress at a checkpoint boundary.
use std::fmt::Display;
use std::fmt::Formatter;

/// Snapshot of training progress at a checkpoint boundary.
pub struct Checkpoint {
    epoch: usize,
    nodes: usize,
    infos: usize,
    rate: f64,
}

impl Checkpoint {
    pub fn new(epoch: usize, nodes: usize, infos: usize, rate: f64) -> Self {
        Self {
            epoch,
            nodes,
            infos,
            rate,
        }
    }

    pub fn epoch(&self) -> usize {
        self.epoch
    }

    pub fn nodes(&self) -> usize {
        self.nodes
    }

    pub fn infos(&self) -> usize {
        self.infos
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }
}

impl Display for Checkpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<20}{:<20}{:<20}{:<20}",
            format!("batch {}", self.epoch),
            format!("nodes {}", self.nodes),
            format!("infos {}", self.infos),
            format!("I/sec {:.1}", self.rate),
        )
    }
}
