//! Outcome of one hand against Slumbot — produced by
//! [`Session::play`](crate::Session::play), consumed by
//! [`Benchmark`](crate::Benchmark) for per-position stats and totals.
use kicker::Turn;

#[derive(Debug, Clone)]
pub struct HandResult {
    pub winnings_bb: f64,
    pub hero: Turn,
}
