//! Run mode: a fixed sample, or until someone stops us.

use pokerkit::Adversary;
use std::time::Duration;

/// How long a benchmark runs and how hard it pushes.
///
/// The two modes exist for different jobs.
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Fixed(usize),
    Continuous,
}

/// Ceiling on continuous-mode exponential backoff. Five minutes is long enough to ride out an
/// API restart without abandoning the run.
const CEILING: u64 = 300;

impl Mode {
    /// Fold the `--hands` / `--continuous` flag pair into one value.
    pub fn new(hands: usize, continuous: bool) -> Self {
        if continuous { Self::Continuous } else { Self::Fixed(hands) }
    }

    /// Total hands to play, where that is known up front.
    pub fn target(self) -> Option<usize> {
        match self {
            Self::Fixed(hands) => Some(hands),
            Self::Continuous => None,
        }
    }

    /// Whether the run has drawn its whole sample.
    pub fn done(self, played: usize) -> bool {
        self.target().is_some_and(|hands| played >= hands)
    }

    /// Pause between successful hands.
    pub fn pace(self, who: Adversary) -> Duration {
        match self {
            Self::Fixed(_) => Duration::ZERO,
            Self::Continuous => Duration::from_millis(
                std::env::var(format!("{}_DELAY_MS", who.service().to_uppercase()))
                    .ok()
                    .and_then(|ms| ms.parse().ok())
                    .unwrap_or(500),
            ),
        }
    }

    /// Pause after a failed hand, doubling per consecutive failure. A fixed run reconnects and
    /// retries immediately — it has a sample to finish and an operator watching.
    pub fn backoff(self, errors: u32) -> Duration {
        match self {
            Self::Fixed(_) => Duration::ZERO,
            Self::Continuous => Duration::from_secs(2u64.pow(errors.min(8)).min(CEILING)),
        }
    }
}
