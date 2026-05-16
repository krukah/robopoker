//! Safe subgame solving via world-partitioned belief.
//!
//! Implements the safe subgame solving technique from Brown & Sandholm 2017
//! using world sampling and per-world regret separation. This crate holds
//! only the "World" layer — see `rbp_depth` for leaf (depth-limited)
//! evaluation and `rbp_subgame` for the combined solver.
//!
//! # Overview
//!
//! Standard subgame solving can be exploited: an opponent could deviate from
//! the blueprint before entering the subgame, reaching states the solver
//! didn't account for. Safe subgame solving addresses this by:
//!
//! 1. Computing opponent reach distribution over hand abstractions
//! 2. Partitioning into K "worlds" weighted by reach probability
//! 3. Sampling worlds proportional to belief weights each iteration
//! 4. Running CFR with per-world info sets to prevent exploitation
//!
//! # File layout (one type per file)
//!
//! - `belief` — `Belief`
//! - `world` — `World` (primitive index)
//! - `info` — `WorldInfo`
//! - `profile` — `WorldProfile`
//! - `encoder` — `WorldEncoder`
//! - `partition` — `Partition` trait
//! - `recall` — `CfrRecall`
//! - `restrict` — `WorldRestrict` trait
//! - `secret` — `Secret` type alias
//! - `solver` — `WorldSolver`
//!
//! # References
//!
//! Brown, N., & Sandholm, T. (2019). Superhuman AI for multiplayer poker.
//! Science, 365(6456), 885-890.

mod belief;
mod encoder;
mod info;
mod partition;
mod profile;
mod recall;
mod restrict;
mod secret;
mod solver;
mod world;

pub use belief::*;
pub use encoder::*;
pub use info::*;
pub use partition::*;
pub use profile::*;
pub use recall::*;
pub use restrict::*;
pub use secret::*;
pub use solver::*;
pub use world::*;
