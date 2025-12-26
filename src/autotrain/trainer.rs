//! Session trait - unified training abstraction

use super::*;
use crate::cards::*;
use crate::database::*;
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
        self.pretraining().await;
        log::info!("training blueprint");
        log::info!("press 'Q + ↵' to stop gracefully");
        loop {
            self.step().await;
            self.checkpoint().await.map(|s| log::info!("{}", s));
            if crate::interrupted() {
                log::info!("{}", self.summary().await);
                break;
            }
        }
        self.sync().await;
    }

    async fn pretraining(&self) {
        PreTraining::run(self.client()).await;
    }

    async fn epochs(&self) -> usize {
        self.client().epochs().await
    }
    async fn blueprint(&self) -> usize {
        self.client().blueprint().await
    }
    async fn complete(&self, street: Street) -> bool {
        self.client().clustered(street).await
    }

    async fn status(&self) {
        fn commas(n: usize) -> String {
            n.to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(|c| std::str::from_utf8(c).unwrap())
                .collect::<Vec<_>>()
                .join(",")
        }
        log::info!("┌────────────┬───────────────┐");
        log::info!("│ Street     │ Clustered     │");
        log::info!("├────────────┼───────────────┤");
        for street in Street::all().iter().rev().cloned() {
            let done = self.complete(street).await;
            let mark = if done { "✓" } else { " " };
            log::info!(
                "│ {:?}{} │       {}       │",
                street,
                " ".repeat(10 - format!("{:?}", street).len()),
                mark
            );
        }
        log::info!("├────────────┼───────────────┤");
        log::info!("│ Epoch      │ {:>13} │", commas(self.epochs().await));
        log::info!("│ Blueprint  │ {:>13} │", commas(self.blueprint().await));
        log::info!("└────────────┴───────────────┘");
    }
}
