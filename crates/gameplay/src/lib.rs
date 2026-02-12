//! Poker game engine with state management, action handling, and settlement.
//!
//! This module implements the rules and mechanics of No-Limit Texas Hold'em,
//! tracking game state across betting rounds and resolving showdowns.
//!
//! ## State Representation
//!
//! - [`Game`] — The memoryless present: stacks, pot, board, and active players
//! - [`Partial`] — The remembered past: complete action history for a hand
//! - [`Path`] — Compressed action sequence for tree traversal
//!
//! ## Actions
//!
//! - [`Action`] — A player decision: fold, check, call, or raise
//! - [`Edge`] — A game tree transition with the acting player
//! - [`Odds`] — Pot odds abstraction for bet sizing (1/3, 1/2, 2/3, pot, all-in)
//!
//! ## Resolution
//!
//! - [`Showdown`] — Final hand comparison when multiple players remain
//! - [`Settlement`] — Pot distribution with side-pot handling
//! - [`PnL`] — Profit and loss accounting per player
//!
//! ## Supporting Types
//!
//! - [`Seat`] — Player position and stack at the table
//! - [`Turn`] — Whose action it is and what options they have
//! - [`Arrangement`] — Positional configuration for heads-up or multiway
//! - [`Abstraction`] — Abstract bucket assignment for strategic equivalence
//!
//! ## Information Levels
//!
//! - [`Partial`] — Partial information: hero's cards only (concrete)
//! - [`Perfect`] — Complete information: both players' cards (concrete)
mod abstraction;
mod action;
mod arrangement;
mod edge;
mod game;
mod odds;
mod partial;
mod path;
mod perfect;
mod pnl;
mod recall;
mod seat;
mod settlement;
mod showdown;
mod size;
mod turn;

pub use abstraction::*;
pub use action::*;
pub use arrangement::*;
pub use edge::*;
pub use game::*;
pub use odds::*;
pub use partial::*;
pub use path::*;
pub use perfect::*;
pub use pnl::*;
pub use recall::*;
pub use seat::*;
pub use settlement::*;
pub use showdown::*;
pub use size::*;
pub use turn::*;
