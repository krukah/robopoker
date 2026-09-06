//! Combined safe subgame solving + depth-limited frontiers.
//!
//! A thin composition layer: [`SubGameEncoder`] tags info sets with a world
//! *and* detects frontier chance nodes, so [`SubGameSolver`] runs
//! world-partitioned safety and depth-limited leaf evaluation at once. The
//! `world` and `depth` types are re-exported, so `use subgame::*` suffices.
//!
//! ```text
//! WorldInfo<DepthInfo<I>>              ← info-set wrapping (world tag outside, leaf phase inside)
//!        WorldProfile<DepthView<P>>    ← profile layering (mutable per-world / read-through to blueprint)
//!               DepthGame<G>          ← game wrapping (depth-limited frontier phase)
//! ```
//!
//! Brown, N., & Sandholm, T. (2019). Superhuman AI for multiplayer poker.
//! Science, 365(6456), 885-890.

mod depth;
mod nest;
mod world;
pub use depth::*;
pub use nest::*;
pub use world::*;

mod encoder;
mod hyperparams;
mod solver;

pub use encoder::*;
pub use hyperparams::*;
pub use solver::*;
