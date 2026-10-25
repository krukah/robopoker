use crate::simplex::Simplex;
use pokerkit::*;

/// Per-player snapshot of CFR state at a given epoch.
pub struct Snapshot {
    epoch: usize,
    iterated: Simplex,
    averaged: Simplex,
    regrets: [Utility; 3],
    weights: [Probability; 3],
    payoffs: [Utility; 3],
    visits: [u32; 3],
}
impl Snapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        epoch: usize,
        iterated: Simplex,
        averaged: Simplex,
        regrets: [Utility; 3],
        weights: [Probability; 3],
        payoffs: [Utility; 3],
        visits: [u32; 3],
    ) -> Self {
        Self {
            epoch,
            iterated,
            averaged,
            regrets,
            weights,
            payoffs,
            visits,
        }
    }

    pub fn epoch(&self) -> usize {
        self.epoch
    }

    pub fn iterated(&self) -> &Simplex {
        &self.iterated
    }

    pub fn averaged(&self) -> &Simplex {
        &self.averaged
    }

    pub fn regrets(&self) -> &[Utility; 3] {
        &self.regrets
    }

    pub fn weights(&self) -> &[Probability; 3] {
        &self.weights
    }

    pub fn payoffs(&self) -> &[Utility; 3] {
        &self.payoffs
    }

    pub fn visits(&self) -> &[u32; 3] {
        &self.visits
    }
}

impl Snapshot {}
