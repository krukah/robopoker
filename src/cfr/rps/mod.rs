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
//!
//! The payoff values in the Game implementation can be modified to create
//! asymmetric variants that break away from the standard uniform random optimal strategy.
//! For example, increasing the payoff for Rock beating Scissors would cause the
//! Nash equilibrium to shift toward playing Rock more frequently and Scissors less frequently.
//! This allows testing how CFR adapts to different game structures while maintaining
//! the core RPS dynamics.
//!
//! The RPS game is also useful for testing the convergence properties of different
//! CFR variants and strategies, such as:
//! - External sampling vs internal sampling
//! - Regret matching vs other policy improvement methods
//! - Different weighting schemes (linear, exponential, etc.)
//! - Tree search strategies (MCCFR vs vanilla CFR)

pub use blueprint::Blueprint;

pub mod blueprint;
pub mod edge;
pub mod encoder;
pub mod game;
pub mod info;
pub mod profile;
pub mod trainer;
pub mod turn;
