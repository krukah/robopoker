//! Safe subgame solving via world-partitioned belief.
//!
//! Implements the safe subgame solving technique from Brown & Sandholm 2017 using world
//! sampling and per-world regret separation. This crate holds only the "World" layer — see
//! `horizon` for leaf (depth-limited) evaluation and `subgame` for the combined solver.

mod range;
mod solve;

pub use range::*;
pub use solve::*;
