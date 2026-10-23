//! Depth-limited solving for imperfect-information games.
//!
//! Extends the CFR framework with frontier evaluation via biased continuation strategies (Brown
//! & Sandholm, NeurIPS 2018).
//!
//! At depth-limited frontiers (e.g. street boundaries in poker), instead of a single heuristic
//! value, each player simultaneously chooses from D biased continuation strategies.

#[cfg(test)]
mod fixture;
mod frontier;
mod solve;
mod tests;
mod tree;

pub use frontier::*;
pub use solve::*;
pub use tree::*;
