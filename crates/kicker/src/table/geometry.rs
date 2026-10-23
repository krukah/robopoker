//! Pot geometry — discrete SPR (stack-to-pot ratio) bucket on the infoset.
//!
//! `Edge::Raise(Size::SPR(n,d))` is pot-relative, but the *meaning* of a pot-relative size
//! depends on stack depth. A 300%-pot bet on a 6bb pot at 100bb stack is a 18bb feeler; the
//! same encoded edge on a 60bb pot is functionally identical to all-in.
use crate::GameN;

/// Discrete SPR (stack-to-pot ratio) bucket on the infoset. 4 buckets covering the meaningful
/// range of HU 100bb stack-to-pot ratios.
///
/// value distinction collapses.
///
/// commits.
///
/// odds.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum SPR {
    Committed = 0,
    Low = 1,
    #[default]
    Mid = 2,
    Deep = 3,
}

impl SPR {
    /// Bucket boundaries on the SPR axis. Must stay sorted ascending.
    /// Final bucket (`Deep`) is open-ended above the last threshold.
    /// Listed here for the regime fingerprint to detect drift.
    pub const BOUNDARIES: [f32; 3] = [1.5, 4.0, 10.0];

    /// Bucket from a continuous SPR value.
    pub fn from_spr(spr: f32) -> Self {
        if spr <= Self::BOUNDARIES[0] {
            Self::Committed
        } else if spr <= Self::BOUNDARIES[1] {
            Self::Low
        } else if spr <= Self::BOUNDARIES[2] {
            Self::Mid
        } else {
            Self::Deep
        }
    }

    /// Wire-encoding tag — round-trips through `From<u8>` / `Into<u8>`.
    pub fn tag(self) -> u8 {
        self as u8
    }
}

impl<const P: usize> GameN<P> {
    /// SPR bucket at this game state. Pot is clamped to ≥1 chip to avoid
    /// division by zero in pre-blind edge cases (in practice all decision
    /// points have pot ≥ blinds).
    pub fn geometry(&self) -> SPR {
        let pot = self.pot().max(1);
        SPR::from_spr(self.effective() as f32 / pot as f32)
    }
}

impl From<u8> for SPR {
    fn from(b: u8) -> Self {
        match b {
            0 => Self::Committed,
            1 => Self::Low,
            2 => Self::Mid,
            _ => Self::Deep,
        }
    }
}

impl std::fmt::Display for SPR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Committed => write!(f, "committed"),
            Self::Low => write!(f, "low"),
            Self::Mid => write!(f, "mid"),
            Self::Deep => write!(f, "deep"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_partition_spr_axis() {
        assert_eq!(SPR::from_spr(0.0), SPR::Committed);
        assert_eq!(SPR::from_spr(1.5), SPR::Committed);
        assert_eq!(SPR::from_spr(1.51), SPR::Low);
        assert_eq!(SPR::from_spr(4.0), SPR::Low);
        assert_eq!(SPR::from_spr(4.01), SPR::Mid);
        assert_eq!(SPR::from_spr(10.0), SPR::Mid);
        assert_eq!(SPR::from_spr(10.01), SPR::Deep);
        assert_eq!(SPR::from_spr(100.0), SPR::Deep);
    }

    #[test]
    fn tag_roundtrips() {
        for g in [SPR::Committed, SPR::Low, SPR::Mid, SPR::Deep] {
            assert_eq!(SPR::from(g.tag()), g);
        }
    }
}
