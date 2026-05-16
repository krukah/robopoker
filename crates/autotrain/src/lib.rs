//! Automated training pipeline orchestration.
//!
//! This module manages the complete training workflow, from checking database
//! state through clustering and blueprint generation. Supports both single-machine
//! and distributed training modes.
//!
//! ## Pipeline Stages
//!
//! 1. **Pretraining** — Generate abstractions via hierarchical clustering
//! 2. **Fast mode** — Single-machine MCCFR with in-memory profile
//! 3. **Slow mode** — Distributed workers with PostgreSQL synchronization
//!
//! ## Core Types
//!
//! - [`Trainer`] — Main entry point for training orchestration
//! - [`Mode`] — Training configuration (fast vs slow, clustering vs blueprint)
mod epoch;
mod fast;
mod fingerprint;
mod mode;
mod pretraining;
mod slow;
mod snapshot;
mod trainer;
pub mod workers;

pub use epoch::*;
pub use fast::*;
pub use fingerprint::*;
pub use mode::*;
pub use pretraining::*;
pub use slow::*;
pub use snapshot::*;
pub use trainer::*;
pub use workers::*;

/// Ensures all training-related tables exist.
pub async fn ensure_all(client: &tokio_postgres::Client) {
    use rbp_database::Ensure;
    client.ensure::<rbp_nlhe::NlheProfile>().await;
    client.ensure::<crate::EpochMeta>().await;
    client.ensure::<crate::Snapshot>().await;
    client.ensure::<crate::Fingerprint>().await;
}
