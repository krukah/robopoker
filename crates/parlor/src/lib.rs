//! Async runtime for live poker games.
//!
//! Orchestrates multiplayer sessions, coordinating a [`Room`] coordinator, an
//! [`Actor`] per player, and the [`Engine`] state machine over message-passing
//! channels. Player types (human, AI, network) plug in via [`Player`].
mod actor;
mod context;
mod engine;
mod event;
mod player;
pub mod players;
pub mod records;
#[cfg(feature = "server")]
mod repository;
#[cfg(feature = "server")]
mod room;
mod timer;

pub use actor::*;
pub use context::*;
pub use engine::*;
pub use event::*;
pub use player::*;
pub use players::*;
// Selective re-exports from records to avoid Hand conflict with deuce::Hand
pub use records::Participant;
pub use records::Play;
#[cfg(feature = "server")]
pub use repository::*;
#[cfg(feature = "server")]
pub use room::*;
pub use timer::*;
