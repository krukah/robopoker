//! Idempotent table creation extension on [`Client`].
use crate::Schema;
use crate::measure;
use tokio_postgres::Client;

/// Idempotent table creation. Extends `Client` with `ensure::<S>()`.
#[async_trait::async_trait]
pub trait Ensure {
    async fn ensure<S>(&self)
    where
        S: Schema + Send;
}

#[async_trait::async_trait]
impl Ensure for Client {
    async fn ensure<S>(&self)
    where
        S: Schema + Send,
    {
        measure("ensure", self.batch_execute(S::creates()))
            .await
            .unwrap_or_else(|e| panic!("ensure table {}: {}", S::name(), e));
        measure("ensure_indices", self.batch_execute(S::indices()))
            .await
            .unwrap_or_else(|e| panic!("ensure indices on {}: {}", S::name(), e));
    }
}

#[async_trait::async_trait]
impl Ensure for std::sync::Arc<Client> {
    async fn ensure<S>(&self)
    where
        S: Schema + Send,
    {
        self.as_ref().ensure::<S>().await
    }
}
