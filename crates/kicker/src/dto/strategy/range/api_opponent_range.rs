use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiOpponentRange {
    /// Hole-card-level posterior, normalized to sum to 1. Each entry is the probability that
    /// the subject holds that exact pocket given the observed action history under the
    /// blueprint.
    ///
    /// Naive: the enumeration runs over all of `deck − board`, so it is not conditioned on the
    /// *other* seat's cards.
    pub entries: Vec<ApiRangeEntry>,
    /// How many of the subject's own decisions the posterior conditions on.
    /// `0` means every combo carried an empty reach product, so `entries` is
    /// the uniform prior over `deck − board` — pure card combinatorics, not
    /// a learned range. Clients should say so rather than render it as one.
    pub decisions: usize,
}

impl ApiOpponentRange {
    pub fn entries(&self) -> &[ApiRangeEntry] {
        &self.entries
    }

    pub fn decisions(&self) -> usize {
        self.decisions
    }
}
