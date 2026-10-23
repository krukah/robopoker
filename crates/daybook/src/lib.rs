//! Database pipeline for training artifacts.
//!
//! Bulk data movement between Rust structures and PostgreSQL, optimized for
//! the large-scale writes required during abstraction and blueprint training.
//!
//! Table names are functions, not constants, because the suffix depends on the
//! active regime/version — see the `table!` / `versioned!` / `regime!` macros.

mod connection;
mod leaderboard;
mod tables;
mod training;
mod traits;
pub use connection::*;
pub use leaderboard::*;
pub use tables::*;
pub use training::*;
pub use traits::*;
