//! Rock Paper Scissors serves as a minimal toy example for testing CFR behavior.
//!
//! It provides a simple zero-sum game with perfect recall and simultaneous moves,
//! making it ideal for validating core CFR mechanics like:
//! - Regret accumulation and minimization
//! - Policy computation and convergence
//! - Nash equilibrium discovery
//!
//! The game's small state space and tractable optimal strategy
//! allows us to easily verify that the CFR implementation converges correctly.

mod edge;
mod game;
mod solver;
mod turn;

pub use edge::*;
pub use game::*;
pub use solver::*;
pub use turn::*;
