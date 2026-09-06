//! Automated training pipeline orchestration, from database-state check through
//! clustering to blueprint generation.
//!
//! Pretraining generates abstractions via hierarchical clustering; then either
//! fast mode (single-machine MCCFR, in-memory profile) or slow mode
//! (distributed workers synchronizing through PostgreSQL) runs. [`Trainer`] is
//! the entry point, [`Mode`] the configuration.
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
    use daybook::Ensure;
    client.ensure::<nlhe::NlheProfile>().await;
    client.ensure::<crate::EpochMeta>().await;
    client.ensure::<crate::Snapshot>().await;
    client.ensure::<crate::Fingerprint>().await;
}
