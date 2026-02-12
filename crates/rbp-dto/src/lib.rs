//! Data transfer objects for API communication.
//!
//! Request and response types for the analysis API, serializable via `serde`.
//! These types bridge the gap between the Rust domain model and JSON payloads.
mod request;
mod response;

pub use request::*;
pub use response::*;
