use std::time::Duration;

/// Training progress counters shared across trainer implementations, plus
/// default renderers for stat and summary lines.
pub trait Progress {
    /// CFR epochs completed.
    fn epoch(&self) -> usize;
    /// Game tree nodes visited across all epochs.
    fn nodes(&self) -> usize;
    /// Information sets processed across all epochs.
    fn infos(&self) -> usize;
    /// Wall-clock duration since training started.
    fn elapsed(&self) -> Duration;
    /// Aligned columns with I/sec throughput.
    fn format(&self) -> String {
        let rates = self.infos() as f64 / self.elapsed().as_secs().max(1) as f64;
        format!(
            "{:<20}{:<20}{:<20}{:<20}",
            format!("batch {}", self.epoch()),
            format!("nodes {}", self.nodes()),
            format!("infos {}", self.infos()),
            format!("I/sec {rates:.1}"),
        )
    }

    fn stats(&self) -> String {
        self.format()
    }

    fn summary(&self) -> String {
        format!("training stopped\n{}", self.format())
    }
}
