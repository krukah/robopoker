//! Training result analysis and query interface.
mod api;
mod cli;
pub mod handlers;
mod query;
pub use api::*;
pub use cli::*;
pub use query::*;
