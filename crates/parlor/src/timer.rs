use pokerkit::*;
use std::time::Duration;

/// Configuration for game pacing and timeouts.
#[derive(Debug, Clone, Copy)]
pub struct TimerConfig {
    pub deal_hole: Duration,
    pub deal_board: Duration,
    pub showdown: Duration,
    pub results: Duration,
    pub decision: Duration,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            deal_hole: Duration::from_millis(PACE_DEAL_HOLE),
            deal_board: Duration::from_millis(PACE_DEAL_BOARD),
            showdown: Duration::from_millis(PACE_SHOWDOWN),
            results: Duration::from_millis(PACE_RESULTS),
            decision: Duration::from_millis(PACE_DECISION),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config() {
        let config = TimerConfig::default();
        assert_eq!(config.deal_hole, Duration::from_millis(PACE_DEAL_HOLE));
        assert_eq!(config.deal_board, Duration::from_millis(PACE_DEAL_BOARD));
        assert_eq!(config.showdown, Duration::from_millis(PACE_SHOWDOWN));
        assert_eq!(config.results, Duration::from_millis(PACE_RESULTS));
        assert_eq!(config.decision, Duration::from_millis(PACE_DECISION));
    }
}
