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
//! - [`translate`] — Action translation over finite lattices
//! - [`world`] — World-partitioned belief for safe subgame solving
//! - [`depth`] — Depth-limited solving with biased continuation strategies
//! - [`subgame`] — Safe + depth-limited subgame solving
//! - [`nlhe`] — No-Limit Hold'em solver
//! - [`leduc`] — Leduc poker MCCFR validation
//! - [`kuhn`] — Kuhn poker MCCFR validation
//! - [`rps`] — Rock-Paper-Scissors solver
//!
//! ### Infrastructure
//! - [`database`] — Database pipeline
//! - [`auth`] — Authentication
//! - [`telemetry`] — OpenTelemetry init and metric registry
//!
//! ### Application
//! - [`gameroom`] — Async game coordinator with player implementations and records
//! - [`server`] — Unified backend with analysis API and WebSocket hosting
//! - [`autotrain`] — Training orchestration with distributed workers
//! - [`competition`] — Hand history analysis with AIVAT variance reduction
//! - [`slumbot`] — Slumbot benchmark client
//!
//! ### Validation
//! - [`litmus`] — Strategic litmus tests for blueprint validation

pub use rbp_core        as core;
pub use rbp_transport   as transport;
pub use rbp_cards       as cards;
pub use rbp_mccfr       as mccfr;
pub use rbp_gameplay    as gameplay;
pub use rbp_clustering  as clustering;
pub use rbp_translate   as translate;
pub use rbp_world       as world;
pub use rbp_depth       as depth;
pub use rbp_subgame     as subgame;
pub use rbp_nlhe        as nlhe;
pub use rbp_leduc       as leduc;
pub use rbp_kuhn        as kuhn;
pub use rbp_rps         as rps;
pub use rbp_database    as database;
pub use rbp_auth        as auth;
pub use rbp_telemetry   as telemetry;
pub use rbp_gameroom    as gameroom;
pub use rbp_server      as server;
pub use rbp_autotrain   as autotrain;
pub use rbp_competition as competition;
pub use rbp_slumbot     as slumbot;
pub use rbp_litmus      as litmus;

// Re-export commonly used types at the root
pub use rbp_core::*;
