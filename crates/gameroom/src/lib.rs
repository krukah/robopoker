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
//! - [`Channel`] — Typed message channels for player ↔ room communication
//!
//! ## Events
//!
//! - [`Event`] — Messages from room to player (deal, turn, result)
//! - [`Player`] — Trait for pluggable player implementations
//!
//! ## Submodules
//!
//! - [`records`] — Hand history recording and replay
//! - [`players`] — Concrete player implementations (Fish, Human, AI)
mod actor;
mod channel;
mod context;
mod dealer;
mod engine;
mod event;
mod message;
mod player;
mod protocol;
#[cfg(feature = "database")]
mod repository;
#[cfg(feature = "database")]
mod room;
mod table;
mod timer;

pub mod records;
pub mod players;

pub use actor::*;
pub use channel::*;
pub use context::*;
pub use dealer::*;
pub use engine::*;
pub use event::*;
pub use message::*;
pub use player::*;
pub use protocol::*;
#[cfg(feature = "database")]
pub use repository::*;
#[cfg(feature = "database")]
pub use room::*;
pub use table::*;
pub use timer::*;
pub use players::*;

// Re-export specific records types (not Hand which conflicts with rbp_cards::Hand,
// not Room which conflicts with room::Room)
pub use records::Participant;
pub use records::Play;
pub use records::Replay;
