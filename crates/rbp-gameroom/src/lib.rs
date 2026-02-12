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
mod actor;
mod channel;
mod context;
mod dealer;
mod engine;
mod event;
mod message;
mod player;
mod protocol;
mod repository;
mod room;
mod table;
mod timer;

pub use actor::*;
pub use channel::*;
pub use context::*;
pub use dealer::*;
pub use engine::*;
pub use event::*;
pub use message::*;
pub use player::*;
pub use protocol::*;
pub use repository::*;
pub use room::*;
pub use table::*;
pub use timer::*;
