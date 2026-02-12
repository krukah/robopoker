use rbp_core::*;
use std::time::Duration;
use tokio::time::Instant;

/// Configuration for game timeouts.
#[derive(Debug, Clone, Copy)]
pub struct TimerConfig {
    pub decision: Duration,
    pub showdown: Duration,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            decision: Duration::from_secs(10),
            showdown: Duration::from_secs(SHOWDOWN_TIMEOUT),
        }
    }
}

/// Manages deadline tracking for player decisions and showdown phases.
#[derive(Debug)]
pub struct Timer {
    config: TimerConfig,
    deadline: Option<Instant>,
}

impl Timer {
    pub fn new(config: TimerConfig) -> Self {
        Self {
            config,
            deadline: None,
        }
    }
    pub fn with_defaults() -> Self {
        Self::new(TimerConfig::default())
    }
    pub fn start_decision(&mut self) {
        self.deadline = Some(Instant::now() + self.config.decision);
    }
    pub fn start_showdown(&mut self) {
        self.deadline = Some(Instant::now() + self.config.showdown);
    }
    pub fn clear(&mut self) {
        self.deadline = None;
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
    pub fn expired(&self) -> bool {
        self.deadline.map(|d| Instant::now() >= d).unwrap_or(false)
    }
    pub fn remaining(&self) -> Option<Duration> {
        self.deadline
            .map(|d| d.saturating_duration_since(Instant::now()))
    }
    pub fn decision_timeout(&self) -> Duration {
        self.config.decision
    }
    pub fn showdown_timeout(&self) -> Duration {
        self.config.showdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config() {
        let config = TimerConfig::default();
        assert_eq!(config.decision, Duration::from_secs(10));
        assert_eq!(config.showdown, Duration::from_secs(SHOWDOWN_TIMEOUT));
    }
    #[test]
    fn timer_starts_cleared() {
        let timer = Timer::with_defaults();
        assert!(timer.deadline().is_none());
        assert!(!timer.expired());
    }
    #[test]
    fn timer_sets_deadline() {
        let mut timer = Timer::with_defaults();
        timer.start_decision();
        assert!(timer.deadline().is_some());
        assert!(!timer.expired());
    }
    #[test]
    fn timer_clears() {
        let mut timer = Timer::with_defaults();
        timer.start_decision();
        timer.clear();
        assert!(timer.deadline().is_none());
    }
}
