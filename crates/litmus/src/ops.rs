//! `Ops` trait — the I/O surface litmus needs from a backend.
//!
//! Defined here so `litmus` doesn't need to depend on `portal`
//! (which would cycle: server's HTTP handlers want to use `Litmus`).
//! The server crate provides a concrete impl wrapping its existing
//! `StrategyAPI` and `TrainingAPI`.

use croupier::{ApiGridUsage, ApiStatus, ApiStrategy, Witness};

#[async_trait::async_trait]
pub trait Ops: Send + Sync {
    /// Look up the averaged strategy at a specific (witness) infoset.
    async fn policy(&self, recall: Witness) -> anyhow::Result<Option<ApiStrategy>>;

    /// Aggregate per-(street, edge) frequency view across the blueprint.
    async fn grid_usage(&self) -> anyhow::Result<Vec<ApiGridUsage>>;

    /// Blueprint header info: epoch, infoset count, sum_regret.
    async fn status(&self) -> anyhow::Result<ApiStatus>;
}
