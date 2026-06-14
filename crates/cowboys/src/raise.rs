//! Raise spot — the four coupled fields describing a chip-amount raise
//! in context: how much, against what pot, on which street, at what
//! depth in the betting tree.

use kicker::*;
use pokerkit::*;

use crate::*;

/// Bundle of `(chips, pot, street, depth)` describing a raise spot.
/// Passed as a single arg to [`Size::translate`] and convertible into
/// [`Size`] (snap to canonical grid).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Raise {
    chips: Chips,
    pot: Chips,
    street: Street,
    depth: usize,
}

impl Raise {
    pub fn new(chips: Chips, pot: Chips, street: Street, depth: usize) -> Self {
        Self {
            chips,
            pot,
            street,
            depth,
        }
    }

    pub fn chips(&self) -> Chips {
        self.chips
    }

    pub fn pot(&self) -> Chips {
        self.pot
    }

    pub fn street(&self) -> Street {
        self.street
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

/// Snap a raise onto the canonical [`Size`] grid (deterministic, allocation-
/// free, rng-free).
///
/// **Not** byte-equivalent to `Size::translate(raise, &Translation::Snap, _)`:
/// the opening (BB-relative) branch uses integer truncation (`chips /
/// B_BLIND`) for speed, while `translate` uses float distance. They agree
/// for chip amounts that land near a canonical anchor; they can disagree
/// at boundaries where integer division rounds toward zero. Use
/// `Size::translate` when you need the rounding semantics of the encoder.
impl From<Raise> for Size {
    fn from(raise: Raise) -> Self {
        match Size::grid(raise.street(), raise.depth()) {
            None => Size::SPR(1, 1),
            Some(Grid::Opening(bbs)) => {
                let target = raise.chips() / pokerkit::B_BLIND;
                bbs.iter()
                    .copied()
                    .min_by_key(|n| (target as i64 - *n as i64).abs())
                    .map_or(Size::BBs(2), Size::BBs)
            }
            Some(Grid::Postflop(idx)) => {
                let target = raise.chips() as Utility / raise.pot() as Utility;
                idx.iter()
                    .map(|&i| RAISES[i])
                    .min_by(|(an, ad), (bn, bd)| {
                        let a = *an as Probability / *ad as Probability;
                        let b = *bn as Probability / *bd as Probability;
                        (target - a).abs().partial_cmp(&(target - b).abs()).unwrap()
                    })
                    .map_or(Size::SPR(1, 1), |(n, d)| Size::SPR(n, d))
            }
        }
    }
}
