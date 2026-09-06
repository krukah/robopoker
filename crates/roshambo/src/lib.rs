//! Rock-Paper-Scissors as a minimal CFR test case — both a validation tool and
//! a reference implementation of the MCCFR trait hierarchy.
//!
//! RPS is a good test bed: 13 nodes total, two-player zero-sum, and a known
//! equilibrium. It is modeled sequentially (P1 then P2) rather than
//! simultaneously so CFR applies. `ASYMMETRIC_UTILITY` skews the Scissors
//! payoff, checking that CFR shifts the equilibrium away from uniform.

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
