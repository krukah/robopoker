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
//! Orthogonal to `world` (safe subgame solving), which acts on info sets while
//! this acts on game states. They compose purely through wrapper types —
//! `DepthGame<G, D>` at the game level, `WorldInfo<DepthInfo<I, D>>` at the
//! info level — and neither references the other's types.

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
