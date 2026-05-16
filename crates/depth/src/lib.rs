//! Depth-limited solving for imperfect-information games.
//!
//! Extends the CFR framework with frontier evaluation via biased
//! continuation strategies (Brown & Sandholm, NeurIPS 2018).
//!
//! At depth-limited frontiers (e.g. street boundaries in poker),
//! instead of a single heuristic value, each player simultaneously
//! chooses from D biased continuation strategies. The CFR solver
//! learns the optimal mix over iterations, providing robust leaf
//! evaluation without solving the full remaining tree.
//!
//! # Composability
//!
//! This crate is orthogonal to `rbp-world` (safe subgame solving):
//!
//! | Crate | Concern | Acts on |
//! |-------|---------|---------|
//! | `rbp-world` | Root safety (opponent range partitioning) | Info sets |
//! | `rbp-depth` | Leaf evaluation (continuation strategies) | Game states |
//!
//! They compose via wrapper types:
//! - `DepthGame<G, D>` wraps the game at the game level
//! - `WorldInfo<DepthInfo<I, D>>` wraps info at the info level
//! - Neither references the other's types
//!
//! # File layout (one type per file)
//!
//! - `continuation` — `Continuation`
//! - `edge` — `DepthEdge`
//! - `payoffs` — `Payoffs` (D×D matrix)
//! - `phase` — `DepthPhase` (Delegate / Frontier / Internal / External)
//! - `game` — `DepthGame`
//! - `info` — `DepthInfo`
//! - `public` — `DepthPublic`
//! - `encoder` — `DepthEncoder`
//! - `view` — `DepthView` (read-only adapter)
//! - `profile` — `DepthProfile` (mutable local)
//! - `sampler` — `DepthSampler` trait
//! - `solver` — `DepthSolver`

mod continuation;
mod edge;
mod encoder;
mod game;
mod hyperparams;
mod info;
mod payoffs;
mod phase;
mod profile;
mod public;
mod sampler;
mod solver;
mod tests;
mod view;

pub use continuation::*;
pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use hyperparams::*;
pub use info::*;
pub use payoffs::*;
pub use phase::*;
pub use profile::*;
pub use public::*;
pub use sampler::*;
pub use solver::*;
pub use view::*;
