//! Card representation, hand evaluation, and strategic abstraction primitives.
//!
//! This module provides the foundational types for representing poker hands and
//! computing their relative strength. All representations are optimized for
//! bijective encoding and fast bitwise operations.
//!
//! ## Core Types
//!
//! - [`Card`] — A single card as a `(Rank, Suit)` tuple encoded in one byte
//! - [`Hand`] — An unordered set of cards as a 64-bit bitmask
//! - [`Hole`] — A player's two private cards
//! - [`Board`] — The community cards (flop, turn, river)
//! - [`Deck`] — A shuffled collection for dealing
//!
//! ## Evaluation
//!
//! - [`Evaluator`] — Lookup-table hand evaluator, arguably the fastest around
//! - [`Strength`] — Evaluated hand ranking with kicker resolution
//! - [`Ranking`] — Hand category (high card through straight flush)
//!
//! ## Abstraction
//!
//! - [`Observation`] — A strategically-equivalent game state (hole + board + street)
//! - [`Isomorphism`] — Canonical representative under suit permutation
//! - [`Permutation`] — Suit relabeling for equivalence class reduction
//!
//! ## Street Progression
//!
//! [`Street`] encodes the four betting rounds: preflop → flop → turn → river.
//! Each street determines board visibility and abstraction granularity.
mod board;
mod card;
mod deck;
mod evaluator;
mod hand;
mod hands;
mod hole;
mod isomorphism;
mod isomorphisms;
mod kicks;
mod observation;
mod observations;
mod permutation;
mod rank;
mod ranking;
mod street;
mod strength;
mod suit;

pub use board::*;
pub use card::*;
pub use deck::*;
pub use evaluator::*;
pub use hand::*;
pub use hands::*;
pub use hole::*;
pub use isomorphism::*;
pub use isomorphisms::*;
pub use kicks::*;
pub use observation::*;
pub use observations::*;
pub use permutation::*;
pub use rank::*;
pub use ranking::*;
pub use street::*;
pub use strength::*;
pub use suit::*;
