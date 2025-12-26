//! This module provides traits for implementing Counterfactual Regret Minimization (CFR).
//!
//! The traits define the core abstractions needed for CFR:
//! - `Turn`: Represents a player's turn in the game
//! - `Edge`: Represents actions/decisions available at each state
//! - `Game`: Defines the game rules and structure
//! - `Info`: Represents information sets that group indistinguishable states
//! - `Profile`: Maintains strategy profiles and regret values (sync)
//! - `Encoder`: Handles encoding/decoding of game states
//! - `Blueprint`: Implements the CFR training algorithm (sync)
//! - `Trainer`: Unified async training interface for both sync and async implementations
//!
//! These traits allow implementing CFR for different game types while sharing common infrastructure.

mod blueprint;
mod edge;
mod encoder;
mod game;
mod info;
mod profile;
mod turn;

pub use blueprint::*;
pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use info::*;
pub use profile::*;
pub use turn::*;
