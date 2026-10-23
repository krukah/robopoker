//! Async runtime for live poker games.
//!
//! Orchestrates multiplayer sessions, coordinating a [`Room`] coordinator, an
//! [`Actor`] per player, and the [`Engine`] state machine over message-passing
//! channels. Player types (human, AI, network) plug in via [`Player`].
pub mod players;
pub mod records;
mod seat;
mod shell;
mod table;

pub use players::*;
// Selective re-exports from records to avoid Hand conflict with deuce::Hand
pub use records::Participant;
pub use records::Play;
pub use seat::*;
pub use shell::*;
pub use table::*;
