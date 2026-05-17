//! Session trait - unified training abstraction
use rbp_mccfr::Checkpoint;
use std::sync::Arc;
use tokio_postgres::Client;
use tracing::Instrument;

/// Unified training session interface.
/// Both fast and slow modes implement this for polymorphic training loops.
#[async_trait::async_trait]
pub trait Trainer: Send + Sync + Sized {
    /// Database client for persistence operations.
    fn client(&self) -> &Arc<Client>;
    /// Sync in-memory state to database on graceful exit.
    async fn sync(self);
    /// Run one training iteration.
    async fn step(&mut self);
    /// Get current epoch count.
    async fn epoch(&self) -> usize;
    /// Get final summary on completion.
    async fn summary(&self) -> String;
    /// Get training statistics if checkpoint interval has elapsed.
    async fn checkpoint(&self) -> Option<Checkpoint>;
    /// Periodically flush in-memory state to database. Default: no-op.
    async fn flush(&mut self) {}
    /// Label used on training metrics to distinguish session types
    /// (e.g. `fast`, `slow`). Override in each implementor.
    fn session_type(&self) -> &'static str {
        "generic"
    }

    async fn train(mut self) {
        tracing::info!(
            session_type = self.session_type(),
            regime = %rbp_core::regime(),
            "training blueprint"
        );
        tracing::info!("press 'Q + ↵' to stop gracefully");
        let labels = [
            rbp_telemetry::KeyValue::new("session_type", self.session_type()),
            rbp_telemetry::KeyValue::new("regime", format!("{}", rbp_core::regime())),
        ];
        let metrics = rbp_telemetry::metrics::get();
        let mut last_nodes = 0usize;
        let mut last_infos = 0usize;
        loop {
            let epoch = self.epoch().await;
            let step_span =
                tracing::info_span!("mccfr.step", session_type = self.session_type(), epoch,);
            self.step().instrument(step_span).await;
            metrics.mccfr_steps.add(1, &labels);
            self.checkpoint()
                .await
                .inspect(|cp| {
                    tracing::info!(nodes = cp.nodes(), infos = cp.infos(), "checkpoint: {}", cp)
                })
                .inspect(|cp| {
                    metrics
                        .mccfr_nodes
                        .add(cp.nodes().saturating_sub(last_nodes) as u64, &labels)
                })
                .inspect(|cp| {
                    metrics
                        .mccfr_infos
                        .add(cp.infos().saturating_sub(last_infos) as u64, &labels)
                })
                .inspect(|cp| {
                    last_nodes = cp.nodes();
                    last_infos = cp.infos();
                });
            let flush_span =
                tracing::info_span!("mccfr.flush", session_type = self.session_type(), epoch,);
            self.flush().instrument(flush_span).await;
            if rbp_core::interrupted() {
                tracing::info!("{}", self.summary().await);
                break;
            }
        }
        self.sync().await;
    }
}
