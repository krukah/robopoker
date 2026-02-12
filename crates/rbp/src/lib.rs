//! Poker solver toolkit for game-theoretically optimal strategies.
//!
//! This facade crate re-exports all public rbp crates for convenient access.
//!
//! ## Crate Organization
//!
//! ### Core Types
//! - [`core`] — Type aliases, constants, DTOs, and shared traits
//! - [`transport`] — Optimal transport (Sinkhorn, EMD)
//! - [`cards`] — Card primitives and hand evaluation
//! - [`mccfr`] — Game-agnostic CFR framework
//!
//! ### Domain Logic
//! - [`gameplay`] — Poker game engine
//! - [`clustering`] — K-means abstraction
//! - [`nlhe`] — No-Limit Hold'em solver
//!
//! ### Infrastructure
//! - [`database`] — Database pipeline
//! - [`auth`] — Authentication
//!
//! ### Application
//! - [`gameroom`] — Async game coordinator with player implementations and records
//! - [`server`] — Unified backend with analysis API and WebSocket hosting
//! - [`autotrain`] — Training orchestration with distributed workers

pub use rbp_core        as core;
pub use rbp_transport   as transport;
pub use rbp_cards       as cards;
pub use rbp_mccfr       as mccfr;
pub use rbp_gameplay    as gameplay;
pub use rbp_clustering  as clustering;
pub use rbp_nlhe        as nlhe;
pub use rbp_database    as database;
pub use rbp_auth        as auth;
pub use rbp_gameroom    as gameroom;
pub use rbp_server      as server;
pub use rbp_autotrain   as autotrain;

// Re-export commonly used types at the root
pub use rbp_core::*;
