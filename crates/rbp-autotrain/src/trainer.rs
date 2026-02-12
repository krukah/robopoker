//! Session trait - unified training abstraction
use std::sync::Arc;
use tokio_postgres::Client;

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
    async fn checkpoint(&self) -> Option<String>;

    async fn train(mut self) {
        log::info!("training blueprint");
        log::info!("press 'Q + ↵' to stop gracefully");
        loop {
            self.step().await;
            self.checkpoint().await.map(|s| log::info!("{}", s));
            if rbp_core::interrupted() {
                log::info!("{}", self.summary().await);
                break;
            }
        }
        self.sync().await;
    }
}
