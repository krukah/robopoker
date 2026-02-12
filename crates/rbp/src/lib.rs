//! Poker solver toolkit for game-theoretically optimal strategies.
//!
//! This facade crate re-exports all public rbp crates for convenient access.
//!
//! ## Crate Organization
//!
//! ### Core Types
//! - [`core`] — Type aliases, constants, and shared traits
//! - [`transport`] — Optimal transport (Sinkhorn, EMD)
//! - [`cards`] — Card primitives and hand evaluation
//! - [`mccfr`] — Game-agnostic CFR framework
//!
//! ### Domain Logic
//! - [`gameplay`] — Poker game engine
//! - [`dto`] — API request/response types
//! - [`clustering`] — K-means abstraction
//! - [`nlhe`] — No-Limit Hold'em solver
//!
//! ### Infrastructure
//! - [`database`] — Database pipeline
//! - [`auth`] — Authentication
//! - [`records`] — Hand history
//! - [`workers`] — Distributed training
//!
//! ### Application
//! - [`gameroom`] — Async game coordinator
//! - [`players`] — Player implementations
//! - [`analysis`] — Query API
//! - [`hosting`] — WebSocket server
//! - [`server`] — Unified backend
//! - [`autotrain`] — Training orchestration

pub use rbp_core        as core;
pub use rbp_transport   as transport;
pub use rbp_cards       as cards;
pub use rbp_mccfr       as mccfr;
pub use rbp_gameplay    as gameplay;
pub use rbp_dto         as dto;
pub use rbp_clustering  as clustering;
pub use rbp_nlhe        as nlhe;
pub use rbp_database    as database;
pub use rbp_auth        as auth;
pub use rbp_records     as records;
pub use rbp_workers     as workers;
pub use rbp_gameroom    as gameroom;
pub use rbp_players     as players;
pub use rbp_analysis    as analysis;
pub use rbp_hosting     as hosting;
pub use rbp_server      as server;
pub use rbp_autotrain   as autotrain;

// Re-export commonly used types at the root
pub use rbp_core::*;
