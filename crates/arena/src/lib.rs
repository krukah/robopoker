//! Hand history analysis with AIVAT variance reduction.
//!
//! Provides statistical evaluation of player performance from stored hand
//! histories, using the AIVAT technique to reduce variance by 10-44x.
//!
//! ## Modules
//!
//! - [`replay`] — Reconstruct `Game` states from database records
//! - `metrics` — Aggregate statistics and derived poker metrics
//! - `aivat` — AIVAT variance reduction estimator
//! - `repository` — Bulk database queries for evaluation
mod aivat;
mod correction;
mod metrics;
mod replay;
mod repository;
pub use aivat::*;
pub use correction::*;
pub use metrics::*;
pub use replay::*;
pub use repository::*;
