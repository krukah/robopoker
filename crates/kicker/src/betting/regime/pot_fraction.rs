use pokerkit::*;

/// Pot-fraction axis. Used for postflop spots and preflop 3-bet+, where
/// the canonical lattice is `Size::SPR(_, _)` evaluated as `n / d`.
pub struct PotFraction;
impl Axis for PotFraction {}
