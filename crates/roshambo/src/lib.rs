//! Rock-Paper-Scissors as a minimal CFR test case.
//!
//! This crate implements RPS using the MCCFR framework, serving as both
//! a validation tool and a reference implementation for the trait hierarchy.
//!
//! # Why RPS?
//!
//! RPS is ideal for testing CFR because:
//! - **Small state space** — Only 13 nodes total (root + 3×P1 + 9×terminal)
//! - **Known equilibrium** — Uniform mixed strategy is Nash (with asymmetric payoffs, it shifts)
//! - **Two-player zero-sum** — Perfect for CFR's theoretical guarantees
//! - **Sequential structure** — P1 moves, then P2 moves (for CFR, not simultaneous)
//!
//! # Asymmetric Payoffs
//!
//! The implementation uses `ASYMMETRIC_UTILITY` to make Scissors worth more/less,
//! testing that CFR correctly shifts equilibrium away from uniform.

mod edge;
mod encoder;
mod game;
pub mod simplex;
mod solver;
mod turn;

pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use solver::*;
pub use turn::*;
