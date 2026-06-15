//! Combined safe subgame solving + depth-limited frontiers.
//!
//! This crate is a thin composition layer over world-partitioned safe subgame solving (safe subgame
//! solving) and depth-limited leaf evaluation (depth-limited leaf evaluation). It provides
//! the combined solver that uses both techniques simultaneously.
//!
//! # Layering
//!
//! ```text
//! WorldInfo<DepthInfo<I>>              ← info-set wrapping (world tag outside, leaf phase inside)
//!        WorldProfile<DepthView<P>>    ← profile layering (mutable per-world / read-through to blueprint)
//!               DepthGame<G>          ← game wrapping (depth-limited frontier phase)
//! ```
//!
//! # Contents
//!
//! - [`SubGameEncoder`] — Tags info sets with world AND detects frontier chance nodes
//! - [`SubGameSolver`] — Combined solver using both safety and depth-limiting
//!
//! Types from `worldview` and `horizon` are re-exported for convenience so
//! that downstream callers only need `use subgame::*`.
//!
//! # References
//!
//! Brown, N., & Sandholm, T. (2019). Superhuman AI for multiplayer poker.
//! Science, 365(6456), 885-890.

mod depth;
mod world;
pub use depth::*;
pub use world::*;

mod encoder;
mod hyperparams;
mod solver;

pub use encoder::*;
pub use hyperparams::*;
pub use solver::*;
