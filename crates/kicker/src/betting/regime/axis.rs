//! Nlhe axis markers for typed action translation.
//!
//! Both axes are non-negative; chips and pot fractions cannot be negative in any well-formed
//! game state. This permits the pseudo-harmonic translators on either axis.

use pokerkit::*;

/// Big-blind-relative axis. Used for preflop opening spots, where the
/// canonical lattice is `Size::BBs(_)`.
pub struct BB;

impl Axis for BB {}
