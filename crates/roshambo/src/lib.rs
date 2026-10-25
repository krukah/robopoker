//! Rock-Paper-Scissors as a minimal CFR test case — both a validation tool and a reference
//! implementation of the MCCFR trait hierarchy.
//!
//! RPS is a good test bed: 13 nodes total, two-player zero-sum, and a known equilibrium. It is
//! modeled sequentially (P1 then P2) rather than simultaneously so CFR applies.

mod snapshot;
pub use snapshot::*;
mod edge;
mod encoder;
mod game;
mod nash;
pub mod simplex;
mod solver;
mod turn;

pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use nash::*;
pub use turn::*;
