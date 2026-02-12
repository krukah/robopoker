use super::*;
use rbp_cards::*;
use rbp_core::*;

/// Abstract bet sizing for Edge::Raise.
///
/// Two interpretation modes:
/// - `SPR(n, d)`: Pot-relative sizing as fraction n/d (e.g., `SPR(1, 2)` = half pot)
/// - `BBs(n)`: BB-relative sizing for preflop opens (e.g., `BBs(3)` = 3BB)
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Size {
    SPR(Chips, Chips),
    BBs(Chips),
}

impl Size {
    /// Converts Size to chip amount.
    pub fn into_chips(self, pot: Chips) -> Chips {
        match self {
            Self::SPR(n, d) => (pot as Utility * n as Utility / d as Utility) as Chips,
            Self::BBs(n) => n * rbp_core::B_BLIND,
        }
    }
    /// Snaps a chip amount to the nearest canonical size.
    /// At opening spots, snaps to BB; otherwise pot-relative.
    pub fn from_chips(
        chips: Chips,
        pot: Chips,
        opening: bool,
        street: Street,
        depth: usize,
    ) -> Self {
        let raises = Self::raises(street, depth);
        if opening {
            Self::nearest_bb(chips, raises)
        } else {
            Self::nearest_pot(chips, pot, raises)
        }
    }
    fn nearest_bb(chips: Chips, raises: &[Self]) -> Self {
        let target = chips / rbp_core::B_BLIND;
        raises
            .iter()
            .filter_map(|s| match s {
                Self::BBs(n) => Some((*n, *s)),
                Self::SPR(..) => None,
            })
            .min_by_key(|(n, _)| (target as i64 - *n as i64).abs())
            .map(|(_, s)| s)
            .unwrap_or(Self::BBs(2))
    }
    fn nearest_pot(chips: Chips, pot: Chips, raises: &[Self]) -> Self {
        let target = chips as Utility / pot as Utility;
        raises
            .iter()
            .filter_map(|s| match s {
                Self::SPR(n, d) => Some((*n as Probability / *d as Probability, *s)),
                Self::BBs(_) => None,
            })
            .min_by(|(a, _), (b, _)| (target - a).abs().partial_cmp(&(target - b).abs()).unwrap())
            .map(|(_, s)| s)
            .unwrap_or(Self::SPR(1, 1))
    }
    /// Converts to Odds for interop with code expecting Odds type.
    /// BBs variant returns synthetic n:1 odds.
    pub fn odds(self) -> Odds {
        match self {
            Self::SPR(n, d) => Odds::new(n, d),
            Self::BBs(n) => Odds::new(n, 1),
        }
    }
    /// Returns the equivalent SPR form for backwards compatibility.
    /// BBs(n) maps to SPR(n, 1), SPR stays unchanged.
    pub fn as_spr(self) -> Self {
        match self {
            Self::BBs(n) => Self::SPR(n, 1),
            spr => spr,
        }
    }
    /// Returns available raise sizes for a given street and depth.
    /// This is the **single source of truth** for which betting edges exist.
    pub fn raises(street: Street, depth: usize) -> &'static [Self] {
        if depth > rbp_core::MAX_RAISE_REPEATS {
            return &[];
        }
        match (street, depth) {
            (Street::Pref, 0) => &Self::PREF_0,
            (Street::Pref, 1) => &Self::PREF_1,
            (Street::Pref, _) => &Self::PREF_N,
            (Street::Flop, 0) => &Self::FLOP_0,
            (Street::Flop, 1) => &Self::FLOP_1,
            (Street::Flop, _) => &Self::FLOP_N,
            (Street::Turn, 0) => &Self::TURN_0,
            (Street::Turn, _) => &Self::TURN_N,
            (Street::Rive, 0) => &Self::RIVE_0,
            (Street::Rive, 1) => &Self::RIVE_1,
            (Street::Rive, _) => &Self::RIVE_N,
        }
    }
}

/// Blinds values used in preflop opening (must fit in u8 6-9).
const BLINDS_GRID: [Chips; 4] = [2, 3, 4, 8];
/// Pot-relative sizes actually used in raises (must fit in u8 10-15, Path uses 4-bit encoding).
const SPR_GRID: [Size; 6] = [
    Size::SPR(1, 3), // 0.33 pot
    Size::SPR(1, 2), // 0.50 pot
    Size::SPR(2, 3), // 0.66 pot
    Size::SPR(1, 1), // 1.00 pot
    Size::SPR(3, 2), // 1.50 pot
    Size::SPR(2, 1), // 2.00 pot
];

#[rustfmt::skip]
impl Size {
    const PREF_0: [Self; 4] = [Self::BBs(2), Self::BBs(3), Self::BBs(4), Self::BBs(8)];             // Preflop depth=0: BB opens
    const PREF_1: [Self; 3] = [Self::SPR(1, 1), Self::SPR(3, 2), Self::SPR(2, 1)];                  // Preflop depth=1: 3-bet sizing (1x, 1.5x, 2x pot)
    const PREF_N: [Self; 2] = [Self::SPR(1, 1), Self::SPR(2, 1)];                                   // Preflop depth=2+: 4-bet+ (1x, 2x pot)
    const FLOP_0: [Self; 4] = [Self::SPR(1, 3), Self::SPR(1, 2), Self::SPR(1, 1), Self::SPR(2, 1)]; // Flop depth=0: probe bet (1/3 instead of 1/4 due to encoding limit)
    const FLOP_1: [Self; 3] = [Self::SPR(2, 3), Self::SPR(1, 1), Self::SPR(3, 2)];                  // Flop depth=1: after first raise (2/3x, 1x, 1.5x)
    const FLOP_N: [Self; 2] = [Self::SPR(1, 1), Self::SPR(3, 2)];                                   // Flop depth=2+: simplified (1x, 1.5x pot)
    const TURN_0: [Self; 4] = [Self::SPR(1, 3), Self::SPR(2, 3), Self::SPR(1, 1), Self::SPR(2, 1)]; // Turn depth=0: geometric sizing for river setup
    const TURN_N: [Self; 2] = [Self::SPR(1, 1), Self::SPR(3, 2)];                                   // Turn depth=1+: simplified (1x, 1.5x pot)
    const RIVE_0: [Self; 4] = [Self::SPR(1, 3), Self::SPR(1, 2), Self::SPR(1, 1), Self::SPR(2, 1)]; // River depth=0: full range including overbets
    const RIVE_1: [Self; 3] = [Self::SPR(2, 3), Self::SPR(1, 1), Self::SPR(2, 1)];                  // River depth=1: raise (2/3x, 1x, 2x pot)
    const RIVE_N: [Self; 1] = [Self::SPR(1, 1)];                                                    // River depth=2+: minimal
}

impl From<Odds> for Size {
    fn from(odds: Odds) -> Self {
        Self::SPR(odds.numer(), odds.denom())
    }
}

/// u8 bijection: encodes to values 6-15 to avoid collision with Edge's 1-5.
/// Layout: 6-9 = BBs(BLINDS_GRID[i-6]), 10-15 = SPR(SPR_GRID[i-10])
impl From<Size> for u8 {
    fn from(size: Size) -> Self {
        match size {
            Size::BBs(n) => {
                6 + BLINDS_GRID
                    .iter()
                    .position(|&b| b == n)
                    .expect("invalid blinds value") as u8
            }
            Size::SPR(..) => {
                10 + SPR_GRID
                    .iter()
                    .position(|&s| s == size)
                    .expect("invalid SPR value") as u8
            }
        }
    }
}
impl From<u8> for Size {
    fn from(value: u8) -> Self {
        match value {
            6..=9 => Self::BBs(BLINDS_GRID[value as usize - 6]),
            10..=15 => SPR_GRID[value as usize - 10],
            _ => panic!("invalid size encoding: {}", value),
        }
    }
}
/// u64 bijection: tag in low bits, value in high bits
impl From<Size> for u64 {
    fn from(size: Size) -> Self {
        match size {
            Size::BBs(n) => (1 << 19) | ((n as u64) << 3),
            Size::SPR(n, d) => ((n as u64) << 3) | ((d as u64) << 11),
        }
    }
}
impl From<u64> for Size {
    fn from(value: u64) -> Self {
        if value & (1 << 19) != 0 {
            Self::BBs(((value >> 3) & 0xFF) as Chips)
        } else {
            Self::SPR(
                ((value >> 3) & 0xFF) as Chips,
                ((value >> 11) & 0xFF) as Chips,
            )
        }
    }
}
impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SPR(n, d) => write!(f, "{}:{}", n, d),
            Self::BBs(n) => write!(f, "{}bb", n),
        }
    }
}
impl TryFrom<&str> for Size {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(bb) = s.strip_suffix("bb") {
            return bb
                .parse::<Chips>()
                .map(Self::BBs)
                .map_err(|e| anyhow::anyhow!("invalid bb format: {}", e));
        }
        if let Some((n, d)) = s.split_once(':') {
            let n = n
                .parse::<Chips>()
                .map_err(|e| anyhow::anyhow!("invalid SPR numerator: {}", e))?;
            let d = d
                .parse::<Chips>()
                .map_err(|e| anyhow::anyhow!("invalid SPR denominator: {}", e))?;
            return Ok(Self::SPR(n, d));
        }
        Err(anyhow::anyhow!("invalid size format: {}", s))
    }
}
impl Arbitrary for Size {
    fn random() -> Self {
        use rand::prelude::IndexedRandom;
        let ref mut rng = rand::rng();
        let all_sizes: Vec<Self> = BLINDS_GRID
            .iter()
            .map(|&n| Self::BBs(n))
            .chain(SPR_GRID.iter().copied())
            .collect();
        *all_sizes.choose(rng).expect("sizes empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::MAX_RAISE_REPEATS;
    /// Verify the raises grid returns expected counts per street/depth.
    /// This is the authoritative spec for action abstraction branching factor.
    #[test]
    fn raises_grid_counts() {
        // Preflop: BB opens with BB-relative sizes, then pot-relative
        assert_eq!(Size::raises(Street::Pref, 0).len(), 4); // 2BB, 3BB, 4BB, 8BB
        assert_eq!(Size::raises(Street::Pref, 1).len(), 3); // 1x, 1.5x, 2x pot
        assert_eq!(Size::raises(Street::Pref, 2).len(), 2); // 1x, 2x pot
        assert_eq!(Size::raises(Street::Pref, 3).len(), 2); // 1x, 2x pot
        // Flop: includes probe bets at depth=0
        assert_eq!(Size::raises(Street::Flop, 0).len(), 4); // 1/3, 1/2, 1x, 2x
        assert_eq!(Size::raises(Street::Flop, 1).len(), 3); // 2/3, 1x, 1.5x
        assert_eq!(Size::raises(Street::Flop, 2).len(), 2); // 1x, 1.5x
        // Turn: geometric sizing for river setup
        assert_eq!(Size::raises(Street::Turn, 0).len(), 4); // 1/3, 2/3, 1x, 2x
        assert_eq!(Size::raises(Street::Turn, 1).len(), 2); // 1x, 1.5x
        // River: overbets matter more
        assert_eq!(Size::raises(Street::Rive, 0).len(), 4); // 1/3, 1/2, 1x, 2x
        assert_eq!(Size::raises(Street::Rive, 1).len(), 3); // 2/3, 1x, 2x
        // Beyond MAX_RAISE_REPEATS: empty (no more raises allowed)
        assert_eq!(Size::raises(Street::Pref, MAX_RAISE_REPEATS + 1).len(), 0);
    }
    /// Preflop depth=0 must use BBs variant (BB-relative sizing).
    #[test]
    fn preflop_opening_uses_bbs() {
        for size in Size::raises(Street::Pref, 0) {
            assert!(
                matches!(size, Size::BBs(_)),
                "preflop depth=0 should use BBs, got {:?}",
                size
            );
        }
    }
    /// Post-flop and preflop depth>0 must use SPR variant (pot-relative).
    #[test]
    fn postflop_uses_spr() {
        for street in [Street::Flop, Street::Turn, Street::Rive] {
            for depth in 0..=MAX_RAISE_REPEATS {
                for size in Size::raises(street, depth) {
                    assert!(
                        matches!(size, Size::SPR(..)),
                        "{:?} depth={} should use SPR, got {:?}",
                        street,
                        depth,
                        size
                    );
                }
            }
        }
        for depth in 1..=MAX_RAISE_REPEATS {
            for size in Size::raises(Street::Pref, depth) {
                assert!(
                    matches!(size, Size::SPR(..)),
                    "preflop depth={} should use SPR, got {:?}",
                    depth,
                    size
                );
            }
        }
    }
    /// All sizes returned by raises() must be encodable to u8 and back.
    #[test]
    fn all_raises_are_encodable() {
        for street in [Street::Pref, Street::Flop, Street::Turn, Street::Rive] {
            for depth in 0..=MAX_RAISE_REPEATS {
                for &size in Size::raises(street, depth) {
                    let encoded = u8::from(size);
                    let decoded = Size::from(encoded);
                    assert_eq!(size, decoded, "roundtrip failed for {:?}", size);
                }
            }
        }
    }
    /// u8 bijection: encode then decode preserves value.
    #[test]
    fn bijective_u8() {
        for &n in &BLINDS_GRID {
            let size = Size::BBs(n);
            assert_eq!(size, Size::from(u8::from(size)));
        }
        for &size in &SPR_GRID {
            assert_eq!(size, Size::from(u8::from(size)));
        }
    }
    /// u64 bijection: encode then decode preserves value.
    #[test]
    fn bijective_u64() {
        for &n in &BLINDS_GRID {
            let size = Size::BBs(n);
            assert_eq!(size, Size::from(u64::from(size)));
        }
        for &size in &SPR_GRID {
            assert_eq!(size, Size::from(u64::from(size)));
        }
    }
    /// into_chips: BBs variant multiplies by B_BLIND.
    #[test]
    fn into_chips_bbs() {
        let pot = 100; // ignored for BBs
        assert_eq!(Size::BBs(2).into_chips(pot), 2 * rbp_core::B_BLIND);
        assert_eq!(Size::BBs(3).into_chips(pot), 3 * rbp_core::B_BLIND);
        assert_eq!(Size::BBs(8).into_chips(pot), 8 * rbp_core::B_BLIND);
    }
    /// into_chips: SPR variant multiplies pot by fraction.
    #[test]
    fn into_chips_spr() {
        let pot = 100;
        assert_eq!(Size::SPR(1, 2).into_chips(pot), 50); // half pot
        assert_eq!(Size::SPR(1, 1).into_chips(pot), 100); // full pot
        assert_eq!(Size::SPR(2, 1).into_chips(pot), 200); // 2x pot
    }
    /// from_chips snaps to nearest canonical size.
    #[test]
    fn from_chips_snaps_to_nearest() {
        let pot = 100;
        // Opening spot (preflop depth=0): snaps to BBs
        let size = Size::from_chips(5, pot, true, Street::Pref, 0);
        assert!(matches!(size, Size::BBs(2) | Size::BBs(3)));
        // Non-opening: snaps to SPR
        let size = Size::from_chips(75, pot, false, Street::Flop, 0);
        assert!(matches!(size, Size::SPR(..)));
    }
    /// SPR_GRID must contain all SPR sizes used in any raises() call.
    #[test]
    fn spr_grid_is_complete() {
        let mut all_spr = std::collections::HashSet::new();
        for street in [Street::Pref, Street::Flop, Street::Turn, Street::Rive] {
            for depth in 0..=MAX_RAISE_REPEATS {
                for &size in Size::raises(street, depth) {
                    if matches!(size, Size::SPR(..)) {
                        all_spr.insert(size);
                    }
                }
            }
        }
        for size in all_spr {
            assert!(SPR_GRID.contains(&size), "SPR_GRID missing {:?}", size);
        }
    }
    /// BLINDS_GRID must contain all BB values used in raises().
    #[test]
    fn blinds_grid_is_complete() {
        let mut all_bbs = std::collections::HashSet::new();
        for street in [Street::Pref, Street::Flop, Street::Turn, Street::Rive] {
            for depth in 0..=MAX_RAISE_REPEATS {
                for size in Size::raises(street, depth) {
                    if let Size::BBs(n) = size {
                        all_bbs.insert(*n);
                    }
                }
            }
        }
        for n in all_bbs {
            assert!(BLINDS_GRID.contains(&n), "BLINDS_GRID missing {}", n);
        }
    }
    /// Display format: BBs shows "Nbb", SPR shows ratio format.
    #[test]
    fn display_format() {
        assert_eq!(format!("{}", Size::BBs(3)), "3bb");
        assert_eq!(format!("{}", Size::SPR(1, 2)), "1:2");
        assert_eq!(format!("{}", Size::SPR(1, 1)), "1:1");
        assert_eq!(format!("{}", Size::SPR(3, 2)), "3:2");
        assert_eq!(format!("{}", Size::SPR(2, 1)), "2:1");
        assert_eq!(format!("{}", Size::SPR(1, 3)), "1:3");
        assert_eq!(format!("{}", Size::SPR(2, 3)), "2:3");
    }
    /// String parsing roundtrip: parse(display(size)) == size.
    #[test]
    fn string_roundtrip() {
        for &n in &BLINDS_GRID {
            let size = Size::BBs(n);
            let s = size.to_string();
            assert_eq!(Size::try_from(s.as_str()).unwrap(), size);
        }
        for &size in &SPR_GRID {
            let s = size.to_string();
            assert_eq!(Size::try_from(s.as_str()).unwrap(), size);
        }
    }
}
