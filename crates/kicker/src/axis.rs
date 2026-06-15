//! Nlhe axis markers for typed action translation.
//!
//! Two axes mediate translation between concrete chip amounts and the
//! abstract `Size` lattice:
//!
//! - [`BB`] — big-blind-relative (preflop opens scaled by `B_BLIND`)
//! - [`PotFraction`] — pot-relative (postflop bets scaled by current pot)
//!
//! Both axes are non-negative; chips and pot fractions cannot be negative
//! in any well-formed game state. This permits the pseudo-harmonic
//! translators on either axis.

use pokerkit::*;

/// Big-blind-relative axis. Used for preflop opening spots, where the
/// canonical lattice is `Size::BBs(_)`.
pub struct BB;

impl Axis for BB {}

/// Pot-fraction axis. Used for postflop spots and preflop 3-bet+, where
/// the canonical lattice is `Size::SPR(_, _)` evaluated as `n / d`.
pub struct PotFraction;

impl Axis for PotFraction {}
