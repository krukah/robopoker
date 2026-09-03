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
//! - [`clustering`] — K-means abstraction
//! - [`holdem`] — No-Limit Hold'em solver
//!
//! ### Infrastructure
//! - [`database`] — Database pipeline
//! - [`auth`] — Authentication
//!
//! ### Application
//! - [`gameroom`] — Async game coordinator (includes records, players)
//! - [`server`] — Unified backend (includes topology, hosting)
//! - [`autotrain`] — Training orchestration (includes workers)

pub use bouncer as auth;
pub use daybook as database;
pub use deuce as cards;
pub use forge as autotrain;
pub use kicker as gameplay;
pub use leduc;
pub use lloyd as clustering;
pub use mccfr;
pub use monge as transport;
pub use nlhe as holdem;
pub use parlor as gameroom;
pub use pokerkit as core;
pub use portal as server;
pub use roshambo as rps;
pub use subgame;

// Re-export commonly used types at the root
pub use pokerkit::*;
