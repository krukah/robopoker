//! Async runtime for live poker games.
//!
//! This module orchestrates multiplayer poker sessions, coordinating between
//! the game engine and various player types (human, AI, network) through
//! message-passing channels.
//!
//! ## Architecture
//!
//! - [`Room`] — Game coordinator managing player registration and hand lifecycle
//! - [`Actor`] — Async task wrapper for a single player's decision loop
//! - [`Engine`] — Game state machine driving the hand forward
//!
//! ## Events
//!
//! - [`Event`] — Messages from room to player (deal, turn, result)
//! - [`Player`] — Trait for pluggable player implementations
mod actor;
mod context;
mod engine;
mod event;
mod player;
pub mod players;
pub mod records;
#[cfg(feature = "database")]
mod repository;
#[cfg(feature = "database")]
mod room;
mod timer;

pub use actor::*;
pub use context::*;
pub use engine::*;
pub use event::*;
pub use player::*;
pub use players::*;
// Selective re-exports from records to avoid Hand conflict with rbp_cards::Hand
pub use records::Participant;
pub use records::Play;
#[cfg(feature = "database")]
pub use repository::*;
#[cfg(feature = "database")]
pub use room::*;
pub use timer::*;
