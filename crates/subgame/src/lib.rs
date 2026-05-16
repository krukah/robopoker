//! Combined safe subgame solving + depth-limited frontiers.
//!
//! This crate is a thin composition layer over [`rbp_world`] (safe subgame
//! solving) and [`rbp_depth`] (depth-limited leaf evaluation). It provides
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
//! Types from `rbp_world` and `rbp_depth` are re-exported for convenience so
//! that downstream callers only need `use rbp_subgame::*`.
//!
//! # References
//!
//! Brown, N., & Sandholm, T. (2019). Superhuman AI for multiplayer poker.
//! Science, 365(6456), 885-890.

mod encoder;
mod hyperparams;
mod solver;

pub use encoder::*;
pub use hyperparams::*;
pub use solver::*;
