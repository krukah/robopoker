//! Slow distributed training session
use crate::*;
use crate::workers::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// Slow distributed training using Worker pool.
///
/// Uses Pluribus configuration via [`Pool`].
pub struct SlowSession {
    client: Arc<Client>,
    pool: Pool,
}

impl SlowSession {
    pub async fn new(client: Arc<Client>) -> Self {
        PreTraining::run(&client).await;
        Self {
            pool: Pool::new(client.clone()).await,
            client,
        }
    }
}

#[async_trait::async_trait]
impl Trainer for SlowSession {
    fn client(&self) -> &Arc<Client> {
        &self.client
    }
    async fn step(&mut self) {
        self.pool.step().await;
    }
    async fn epoch(&self) -> usize {
        self.pool.epoch()
    }
    async fn checkpoint(&self) -> Option<String> {
        self.pool.checkpoint()
    }
    async fn summary(&self) -> String {
        self.pool.summary()
    }
    async fn sync(self) {
        // SlowSession writes directly to DB, no sync needed
    }
}
