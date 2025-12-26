use std::time::Duration;

/// Unified trait for training statistics across different trainer implementations.
/// Provides core accessors for epochs, nodes, infos, and elapsed time,
/// with default implementations for formatted stats and summary output.
pub trait TrainingStats {
    fn epoch(&self) -> usize;
    fn nodes(&self) -> usize;
    fn infos(&self) -> usize;
    fn elapsed(&self) -> Duration;
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
