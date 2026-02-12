//! Safe subgame solving with world selection.
//!
//! This module implements the safe subgame solving technique from the Pluribus
//! poker AI. It enables solving from arbitrary game states while maintaining
//! safety guarantees against opponent exploitation.
//!
//! # Overview
//!
//! Standard subgame solving can be exploited: an opponent could deviate from
//! the blueprint before entering the subgame, reaching states the solver
//! didn't account for. Safe subgame solving addresses this by:
//!
//! 1. Computing opponent reach distribution over hand abstractions
//! 2. Clustering into K "worlds" weighted by reach probability
//! 3. Adding a world-selection phase at the subgame root
//! 4. The solver must handle all worlds, preventing exploitation
//!
//! # Components
//!
//! - [`ManyWorlds`] — K-world clustering for the subgame gadget
//! - [`SubGame`] — Game wrapper that adds the world structure
//! - [`SubTurn`], [`SubEdge`] — Turn and edge types for subgames
//! - [`SubInfo`], [`SubPublic`], [`SubSecret`] — Information set types for subgames
//! - [`SubProfile`] — Profile routing between blueprint and local storage
//! - [`SubSolver`] — Complete solver for safe subgame solving
//! - [`SubEncoder`] — Encoder wrapper for subgame-augmented games
//!
//! # References
//!
//! Brown, N., & Sandholm, T. (2019). Superhuman AI for multiplayer poker.
//! Science, 365(6456), 885-890.

mod edge;
mod encoder;
mod game;
mod info;
mod phase;
mod profile;
mod public;
mod secret;
mod solver;
mod turn;
mod worlds;

pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use info::*;
pub use phase::*;
pub use profile::*;
pub use public::*;
pub use secret::*;
pub use solver::*;
pub use turn::*;
pub use worlds::*;
