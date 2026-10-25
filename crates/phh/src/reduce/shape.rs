use super::*;

/// How a logged hand relates to our heads-up tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Ended before a flop. No postflop decisions to compare.
    Preflop,
    /// Three or more players saw the flop. Outside our tree entirely.
    Multiway(usize),
    /// Heads-up on the flop, but blind vs. blind, so postflop action order is
    /// inverted relative to heads-up. Unusable.
    Inverted,
    /// Heads-up on the flop, in heads-up action order.
    HeadsUp(Seats),
}
impl Shape {
    /// The two seats, when the hand reduces.
    pub fn seats(&self) -> Option<Seats> {
        match self {
            Self::HeadsUp(seats) => Some(*seats),
            _ => None,
        }
    }

    /// A short label for census tables.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Preflop => "preflop",
            Self::Multiway(_) => "multiway",
            Self::Inverted => "blind-vs-blind",
            Self::HeadsUp(_) => "heads-up",
        }
    }
}
