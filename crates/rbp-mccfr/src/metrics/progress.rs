use std::time::Duration;

/// Unified trait for training progress across different trainer implementations.
///
/// Provides core accessors for epochs, nodes, infos, and elapsed time,
/// with default implementations for formatted stats and summary output.
///
/// # Required Methods
///
/// - `epoch()` — Number of training iterations completed
/// - `nodes()` — Total game tree nodes visited
/// - `infos()` — Total information sets processed
/// - `elapsed()` — Wall-clock training duration
///
/// # Provided Methods
///
/// - `format()` — Tabular stats with I/sec throughput
/// - `stats()` — Alias for `format()`
/// - `summary()` — Final output with "training stopped" prefix
pub trait Progress {
    /// Number of training iterations (CFR epochs) completed.
    fn epoch(&self) -> usize;
    /// Total game tree nodes visited across all epochs.
    fn nodes(&self) -> usize;
    /// Total information sets processed across all epochs.
    fn infos(&self) -> usize;
    /// Wall-clock duration since training started.
    fn elapsed(&self) -> Duration;
    /// Formats stats as aligned columns with throughput calculation.
    fn format(&self) -> String {
        let rates = self.infos() as f64 / self.elapsed().as_secs().max(1) as f64;
        format!(
            "{:<20}{:<20}{:<20}{:<20}",
            format!("epoch {}", self.epoch()),
            format!("nodes {}", self.nodes()),
            format!("infos {}", self.infos()),
            format!("I/sec {:.1}", rates),
        )
    }
    fn stats(&self) -> String {
        self.format()
    }
    fn summary(&self) -> String {
        format!("training stopped\n{}", self.format())
    }
}
