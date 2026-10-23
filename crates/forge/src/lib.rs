//! Automated training pipeline orchestration, from database-state check through clustering to
//! blueprint generation.
//!
//! Pretraining generates abstractions via hierarchical clustering; then either fast mode
//! (single-machine MCCFR, in-memory profile) or slow mode (distributed workers synchronizing
//! through PostgreSQL) runs. Deep mode trains Deep CFR on NLHE with no database at all. [`Trainer`] is the entry point, [`Mode`] the configuration.

mod tables;
pub(crate) use tables::*;

mod deep;
mod epoch;
mod fast;
mod mode;
mod pretraining;
mod slow;
mod snapshot;
mod trainer;
pub mod workers;

pub use daybook::Fingerprint;
pub use deep::*;
pub use epoch::*;
pub use fast::*;
pub use mode::*;
pub use pretraining::*;
pub use slow::*;
pub use snapshot::*;
pub use trainer::*;
pub use workers::*;
