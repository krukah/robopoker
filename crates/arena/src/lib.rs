//! Hand history analysis with AIVAT variance reduction.
//!
//! Statistical evaluation of player performance from stored hand histories,
//! using AIVAT to cut variance 10-44x.
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
