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

pub use arena as competition;
pub use atlas as world;
pub use bouncer as auth;
pub use cowboys as gameplay;
pub use dragoman as translate;
pub use endgame as subgame;
pub use forge as autotrain;
pub use holdem as nlhe;
pub use horizon as depth;
pub use kicker as cards;
pub use kuhn;
pub use ledger as database;
pub use leduc;
pub use litmus;
pub use lloyd as clustering;
pub use monge as transport;
pub use parlor as gameroom;
pub use pokerkit as core;
pub use portal as server;
pub use regret as mccfr;
pub use roshambo as rps;
pub use spar as slumbot;
pub use vitals as telemetry;

// Re-export commonly used types at the root
pub use pokerkit::*;
