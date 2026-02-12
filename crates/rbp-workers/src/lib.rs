//! Distributed training workers for MCCFR.
//!
//! Worker pool implementation for parallelizing CFR iterations across
//! multiple threads or machines, with PostgreSQL-backed synchronization.
//!
//! ## Core Types
//!
//! - [`Pool`] — Thread pool managing worker lifecycles
//! - [`Worker`] — Individual CFR iteration executor
mod pool;
mod worker;

pub use pool::*;
pub use worker::*;

// Re-export from rbp-database for convenience
pub use rbp_database::Memory;
pub use rbp_database::Record;

// Re-export Progress trait from mccfr
pub use rbp_mccfr::Progress;
