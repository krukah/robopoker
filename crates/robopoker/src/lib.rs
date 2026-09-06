//! Poker solver toolkit for game-theoretically optimal strategies.
//!
//! Re-exports the published library crates under stable aliases.
//!
//! - [`core`] — type aliases, constants, shared traits
//! - [`cards`] — card primitives and hand evaluation
//! - [`transport`] — optimal transport (Sinkhorn, EMD)
//! - [`mccfr`] — game-agnostic CFR framework
//! - [`gameplay`] — poker game engine
//! - [`clustering`] — hierarchical k-means abstraction (EMD)
//! - [`holdem`] — No-Limit Hold'em solver
//! - [`subgame`] — depth-limited and safe subgame solving
//! - [`database`] — PostgreSQL bulk-IO pipeline
//!
//! The auth, game-hosting, server, and training-orchestration crates live
//! in the workspace but are not published; depend on the repository directly
//! to use them.

pub use daybook as database;
pub use deuce as cards;
pub use kicker as gameplay;
pub use lloyd as clustering;
pub use mccfr;
pub use monge as transport;
pub use nlhe as holdem;
pub use pokerkit as core;
pub use subgame;

// Re-export commonly used types at the root
pub use pokerkit::*;
