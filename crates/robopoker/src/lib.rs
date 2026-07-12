//! Poker solver toolkit for game-theoretically optimal strategies.
//!
//! This facade re-exports the published `robopoker` library crates under
//! convenient names. The private application and service crates (live-game
//! coordination, HTTP server, training orchestration, benchmarking) are not
//! re-exported here — depend on them directly from the workspace if you need
//! them.
//!
//! ## Crate Organization
//!
//! ### Core
//! - [`core`] — Type aliases, constants, action translation, shared traits
//! - [`cards`] — Card primitives, hand evaluation, abstraction primitives
//! - [`transport`] — Optimal transport (Sinkhorn, EMD)
//! - [`clustering`] — Generic triangle-inequality-accelerated k-means
//! - [`gameplay`] — Poker game engine
//! - [`mccfr`] — Game-agnostic MCCFR framework
//! - [`subgame`] — Safe + depth-limited subgame solving
//! - [`nlhe`] — No-Limit Hold'em solver
//!
//! ### Infrastructure
//! - [`database`] — PostgreSQL persistence
//! - [`telemetry`] — OpenTelemetry init and metric registry

pub use daybook as database;
pub use deuce as cards;
pub use elkan as clustering;
pub use kicker as gameplay;
pub use mccfr;
pub use monge as transport;
pub use nlhe;
pub use pokerkit as core;
pub use subgame;
pub use vitals as telemetry;

// Re-export commonly used types at the root
pub use pokerkit::*;
