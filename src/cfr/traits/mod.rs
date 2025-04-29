//! This module provides traits for implementing Counterfactual Regret Minimization (CFR).
//!
//! The traits define the core abstractions needed for CFR:
//! - `Turn`: Represents a player's turn in the game
//! - `Edge`: Represents actions/decisions available at each state
//! - `Game`: Defines the game rules and structure
//! - `Info`: Represents information sets that group indistinguishable states
//! - `Profile`: Maintains strategy profiles and regret values
//! - `Encoder`: Handles encoding/decoding of game states
//! - `Trainer`: Implements the CFR training algorithm
//!
//! These traits allow implementing CFR for different game types while sharing common infrastructure.

pub mod edge;
pub mod encoder;
pub mod game;
pub mod info;
pub mod profile;
pub mod trainer;
pub mod turn;

pub use edge::Edge;
pub use encoder::Encoder;
pub use game::Game;
pub use info::Info;
pub use profile::Profile;
pub use trainer::Trainer;
pub use turn::Turn;
