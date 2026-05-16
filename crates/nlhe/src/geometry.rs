//! Pot geometry — discrete SPR (stack-to-pot ratio) bucket on the infoset.
//!
//! `Edge::Raise(Size::SPR(n,d))` is pot-relative, but the *meaning* of a
//! pot-relative size depends on stack depth. A 300%-pot bet on a 6bb pot
//! at 100bb stack is a 18bb feeler; the same encoded edge on a 60bb pot
//! is functionally identical to all-in. Without geometry on the infoset,
//! the abstraction collapses these into the same decision and accumulates
//! regret over both.
//!
//! `Geometry` adds 5 log-spaced SPR buckets to the infoset key. Strategy
//! becomes SPR-aware at the cost of a ~5× infoset blowup (currently
//! 306K → ~1.5M post-reset).
//!
//! Bucket boundaries are part of the regime fingerprint — bumping them is
//! a tree-shape change that must invalidate the blueprint. See
//! `crates/util/src/regime.rs`.
use rbp_core::Chips;
use rbp_gameplay::Game;

/// Discrete pot-geometry bucket. 5 buckets covering the meaningful range
/// of HU 100bb stack-to-pot ratios.
///
/// Reasoning:
/// - **Committed** (SPR ≤ 1.5): every bet sets up a shove; the polar/
///   value distinction collapses.
/// - **Low** (1.5 < SPR ≤ 4): short, polar play; one more bet usually
///   commits.
/// - **Mid** (4 < SPR ≤ 10): the bread-and-butter postflop range.
/// - **Deep** (10 < SPR ≤ 30): implied odds matter; small bets dominate.
/// - **VeryDeep** (SPR > 30): preflop opens, open-call lines.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum Geometry {
    Committed = 0,
    Low = 1,
    #[default]
    Mid = 2,
    Deep = 3,
    VeryDeep = 4,
}

impl Geometry {
    /// Bucket boundaries on the SPR axis. Must stay sorted ascending.
    /// Final bucket (`VeryDeep`) is open-ended above the last threshold.
    /// Listed here for the regime fingerprint to detect drift.
    pub const BOUNDARIES: [f32; 4] = [1.5, 4.0, 10.0, 30.0];

    /// Bucket from a continuous SPR value.
    pub fn from_spr(spr: f32) -> Self {
        if spr <= Self::BOUNDARIES[0] {
            Self::Committed
        } else if spr <= Self::BOUNDARIES[1] {
            Self::Low
        } else if spr <= Self::BOUNDARIES[2] {
            Self::Mid
        } else if spr <= Self::BOUNDARIES[3] {
            Self::Deep
        } else {
            Self::VeryDeep
        }
    }

    /// Bucket from the live game. SPR = effective stack remaining / pot.
    /// Pot is clamped to ≥1 chip to avoid division by zero in pre-blind
    /// edge cases (in practice all decision points have pot ≥ blinds).
    pub fn from_game(game: &Game) -> Self {
        let stacks = game.stacks();
        let eff: Chips = stacks.iter().copied().min().unwrap_or(0);
        let pot = game.pot().max(1);
        Self::from_spr(eff as f32 / pot as f32)
    }

    /// Wire-encoding tag — round-trips through `From<u8>` / `Into<u8>`.
    pub fn tag(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Geometry {
    fn from(b: u8) -> Self {
        match b {
            0 => Self::Committed,
            1 => Self::Low,
            2 => Self::Mid,
            3 => Self::Deep,
            _ => Self::VeryDeep,
        }
    }
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Committed => write!(f, "committed"),
            Self::Low => write!(f, "low"),
            Self::Mid => write!(f, "mid"),
            Self::Deep => write!(f, "deep"),
            Self::VeryDeep => write!(f, "very_deep"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_partition_spr_axis() {
        assert_eq!(Geometry::from_spr(0.0), Geometry::Committed);
        assert_eq!(Geometry::from_spr(1.5), Geometry::Committed);
        assert_eq!(Geometry::from_spr(1.51), Geometry::Low);
        assert_eq!(Geometry::from_spr(4.0), Geometry::Low);
        assert_eq!(Geometry::from_spr(4.01), Geometry::Mid);
        assert_eq!(Geometry::from_spr(10.0), Geometry::Mid);
        assert_eq!(Geometry::from_spr(10.01), Geometry::Deep);
        assert_eq!(Geometry::from_spr(30.0), Geometry::Deep);
        assert_eq!(Geometry::from_spr(30.01), Geometry::VeryDeep);
        assert_eq!(Geometry::from_spr(100.0), Geometry::VeryDeep);
    }

    #[test]
    fn tag_roundtrips() {
        for g in [
            Geometry::Committed,
            Geometry::Low,
            Geometry::Mid,
            Geometry::Deep,
            Geometry::VeryDeep,
        ] {
            assert_eq!(Geometry::from(g.tag()), g);
        }
    }
}
