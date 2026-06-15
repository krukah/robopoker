use super::*;
use deuce::*;
use pokerkit::*;

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
            Self::BBs(n) => n * pokerkit::B_BLIND,
        }
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
    /// Translate a [`Raise`] under a [`Translation`].
    ///
    /// Nlhe-specific adapter over the translation lattice: dispatches to the
    /// appropriate axis (BB-relative for opening spots, pot-fraction
    /// otherwise) via [`Self::grid`] and resolves through the
    /// policy. Returns [`Translated::Snap`] if the resolved anchor is
    /// a canonical `Size`, or [`Translated::Free`] if the policy
    /// injected the observed amount as a fresh anchor.
    pub fn translate<R>(raise: Raise, policy: &Translation, rng: &mut R) -> Translated<Self, Chips>
    where
        R: rand::Rng + ?Sized,
    {
        // Precondition: caller has checked Edge::raises(...).is_empty() and
        // short-circuited the depth-overflow case. The `None` arm covers
        // depth > MAX_RAISE_REPEATS for callers that pass through edgify or
        // other paths without the upstream guard.
        match Self::grid(raise.street(), raise.depth()) {
            None => Translated::Snap(Self::SPR(1, 1)),
            Some(Grid::Opening(bbs)) => Self::translate_axis::<BB, _>(
                raise.chips(),
                raise.chips() as f64 / pokerkit::B_BLIND as f64,
                bbs.iter().map(|&n| (n as f64, Self::BBs(n))),
                policy,
                rng,
            ),
            Some(Grid::Postflop(idx)) => Self::translate_axis::<PotFraction, _>(
                raise.chips(),
                raise.chips() as f64 / raise.pot() as f64,
                idx.iter().map(|&i| {
                    let (n, d) = RAISES[i];
                    (n as f64 / d as f64, Self::SPR(n, d))
                }),
                policy,
                rng,
            ),
        }
    }

    fn translate_axis<A, R>(
        chips: Chips,
        observed: f64,
        pairs: impl IntoIterator<Item = (f64, Self)>,
        policy: &Translation,
        rng: &mut R,
    ) -> Translated<Self, Chips>
    where
        A: Axis,
        R: rand::Rng + ?Sized,
    {
        let lat = pairs.into_iter().collect::<Lattice<A, Self>>();
        let obs = Scalar::<A>::new(observed);
        policy.resolve(obs, &lat, chips, rng)
    }

    /// Returns the axis-typed raise grid for a given street and depth.
    /// `None` if depth exceeds [`pokerkit::MAX_RAISE_REPEATS`].
    pub fn grid(street: Street, depth: usize) -> Option<Grid> {
        if depth > pokerkit::MAX_RAISE_REPEATS {
            None
        } else {
            match pokerkit::regime() {
                pokerkit::Regime::Pluribus => Some(Self::pluribus_grid(street, depth)),
                pokerkit::Regime::Slumbot => Some(Grid::Postflop(SLUMBOT_INDICES)),
            }
        }
    }

    /// Returns the raw RAISES-index slice for a given cell from the
    /// `PLURIBUS_INDICES` table-of-truth in `pokerkit`.
    ///
    /// Returns `&[]` for `(Pref, 0)` since opens are BB-relative; callers
    /// should treat that cell specially via `OPENS`.
    pub const fn indices(street: Street, depth: usize) -> &'static [usize] {
        let row = match street {
            Street::Pref => 0,
            Street::Flop => 3,
            Street::Turn => 6,
            Street::Rive => 9,
        } + if depth > 2 { 2 } else { depth };
        PLURIBUS_INDICES[row]
    }

    fn pluribus_grid(street: Street, depth: usize) -> Grid {
        if matches!(street, Street::Pref) && depth == 0 {
            Grid::Opening(&OPENS)
        } else {
            Grid::Postflop(Self::indices(street, depth))
        }
    }

    /// Returns available raise sizes as an owned vector of [`Size`].
    ///
    /// Allocation is bounded (≤ 5 elements per cell) and the call site
    /// — [`crate::Edge::raises`] — already collects into a `Vec<Edge>`,
    /// so swapping the per-cell static const arrays for this on-the-fly
    /// resolution doesn't add hot-path allocations.
    pub fn raises(street: Street, depth: usize) -> Vec<Self> {
        if depth > pokerkit::MAX_RAISE_REPEATS {
            return Vec::new();
        }
        match pokerkit::regime() {
            pokerkit::Regime::Pluribus => {
                if matches!(street, Street::Pref) && depth == 0 {
                    OPENS.iter().map(|&n| Self::BBs(n)).collect()
                } else {
                    Self::indices(street, depth)
                        .iter()
                        .map(|&i| {
                            let (n, d) = RAISES[i];
                            Self::SPR(n, d)
                        })
                        .collect()
                }
            }
            pokerkit::Regime::Slumbot => SLUMBOT_INDICES
                .iter()
                .map(|&i| {
                    let (n, d) = RAISES[i];
                    Self::SPR(n, d)
                })
                .collect(),
        }
    }
}

impl From<Odds> for Size {
    fn from(odds: Odds) -> Self {
        Self::SPR(odds.numer(), odds.denom())
    }
}

/// u8 bijection: encodes to values 6-19 to avoid collision with Edge's 1-5.
/// Layout: 6-9 = BBs(OPENS[i-6]), 10-19 = SPR(RAISES[i-10])
impl From<Size> for u8 {
    fn from(size: Size) -> Self {
        match size {
            Size::BBs(n) => 6 + OPENS.iter().position(|&b| b == n).expect("invalid blinds value") as u8,
            Size::SPR(n, d) => {
                10 + RAISES
                    .iter()
                    .position(|&(rn, rd)| rn == n && rd == d)
                    .expect("invalid SPR value") as u8
            }
        }
    }
}
impl From<u8> for Size {
    fn from(value: u8) -> Self {
        match value {
            6..=9 => Self::BBs(OPENS[value as usize - 6]),
            10..=19 => {
                let (n, d) = RAISES[value as usize - 10];
                Self::SPR(n, d)
            }
            _ => panic!("invalid size encoding: {value}"),
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
            Self::SPR(((value >> 3) & 0xFF) as Chips, ((value >> 11) & 0xFF) as Chips)
        }
    }
}
impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SPR(n, d) => write!(f, "{n}:{d}"),
            Self::BBs(n) => write!(f, "{n}bb"),
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
                .map_err(|e| anyhow::anyhow!("invalid bb format: {e}"));
        }
        if let Some((n, d)) = s.split_once(':') {
            let n = n
                .parse::<Chips>()
                .map_err(|e| anyhow::anyhow!("invalid SPR numerator: {e}"))?;
            let d = d
                .parse::<Chips>()
                .map_err(|e| anyhow::anyhow!("invalid SPR denominator: {e}"))?;
            return Ok(Self::SPR(n, d));
        }
        Err(anyhow::anyhow!("invalid size format: {s}"))
    }
}
impl Arbitrary for Size {
    fn random() -> Self {
        use rand::prelude::IndexedRandom;
        let ref mut rng = rand::rng();
        let all_sizes: Vec<Self> = OPENS
            .iter()
            .map(|&n| Self::BBs(n))
            .chain(RAISES.iter().map(|&(n, d)| Self::SPR(n, d)))
            .collect();
        *all_sizes.choose(rng).expect("sizes empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pokerkit::MAX_RAISE_REPEATS;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn seeded() -> SmallRng {
        SmallRng::seed_from_u64(0xF00DCAFE)
    }

    /// PLURIBUS_INDICES table-of-truth must agree with `Size::raises` across
    /// every `(street, depth)` cell.
    #[test]
    fn pluribus_indices_match_raises() {
        for street in Street::all() {
            let depths: &[usize] = if street == Street::Pref { &[1, 2] } else { &[0, 1, 2] };
            for &depth in depths {
                let indices = Size::indices(street, depth);
                let raises = Size::raises(street, depth);
                assert_eq!(
                    indices.len(),
                    raises.len(),
                    "{street:?} depth={depth}: PLURIBUS_INDICES len {} != Size::raises len {}",
                    indices.len(),
                    raises.len(),
                );
                for (i, &idx) in indices.iter().enumerate() {
                    let expected = Size::SPR(RAISES[idx].0, RAISES[idx].1);
                    assert_eq!(raises[i], expected, "{street:?} depth={depth}: position {i} mismatch");
                }
            }
        }
    }
    /// Verify the grid returns expected counts per street/depth.
    #[test]
    fn raises_grid_counts() {
        assert_eq!(Size::raises(Street::Pref, 0).len(), 4); // 2BB, 3BB, 4BB, 5BB
        assert_eq!(Size::raises(Street::Pref, 1).len(), 2); // 1:1, 2:1
        assert_eq!(Size::raises(Street::Pref, 2).len(), 1); // 1:1
        assert_eq!(Size::raises(Street::Pref, 3).len(), 1);
        assert_eq!(Size::raises(Street::Flop, 0).len(), 5); // 1/4, 1/2, 3/4, 1:1, 2:1
        assert_eq!(Size::raises(Street::Flop, 1).len(), 2); // 1/2, 1:1
        assert_eq!(Size::raises(Street::Flop, 2).len(), 1); // 1:1
        assert_eq!(Size::raises(Street::Turn, 0).len(), 4); // 1/3, 1/2, 1:1, 2:1
        assert_eq!(Size::raises(Street::Turn, 1).len(), 2); // 1:1, 2:1
        assert_eq!(Size::raises(Street::Turn, 2).len(), 1); // 1:1
        assert_eq!(Size::raises(Street::Rive, 0).len(), 4); // 1/3, 1/2, 1:1, 2:1
        assert_eq!(Size::raises(Street::Rive, 1).len(), 2); // 1:1, 2:1
        assert_eq!(Size::raises(Street::Rive, 2).len(), 1); // 1:1
        assert_eq!(Size::raises(Street::Pref, MAX_RAISE_REPEATS + 1).len(), 0);
    }
    /// Preflop depth=0 must use BBs variant (BB-relative sizing).
    #[test]
    fn preflop_opening_uses_bbs() {
        for size in Size::raises(Street::Pref, 0) {
            assert!(matches!(size, Size::BBs(_)), "preflop depth=0 should use BBs, got {size:?}");
        }
    }
    /// Post-flop and preflop depth>0 must use SPR variant (pot-relative).
    #[test]
    fn postflop_uses_spr() {
        for street in [Street::Flop, Street::Turn, Street::Rive] {
            for depth in 0..=MAX_RAISE_REPEATS {
                for size in Size::raises(street, depth) {
                    assert!(matches!(size, Size::SPR(..)), "{street:?} depth={depth} should use SPR, got {size:?}");
                }
            }
        }
        for depth in 1..=MAX_RAISE_REPEATS {
            for size in Size::raises(Street::Pref, depth) {
                assert!(matches!(size, Size::SPR(..)), "preflop depth={depth} should use SPR, got {size:?}");
            }
        }
    }
    /// All sizes returned by raises() must be encodable to u8 and back.
    #[test]
    fn all_raises_are_encodable() {
        for street in Street::all() {
            for depth in 0..=MAX_RAISE_REPEATS {
                for size in Size::raises(street, depth) {
                    let encoded = u8::from(size);
                    let decoded = Size::from(encoded);
                    assert_eq!(size, decoded, "roundtrip failed for {size:?}");
                }
            }
        }
    }
    /// u8 bijection: encode then decode preserves value.
    #[test]
    fn bijective_u8() {
        for &n in &OPENS {
            let size = Size::BBs(n);
            assert_eq!(size, Size::from(u8::from(size)));
        }
        for &(n, d) in &RAISES {
            let size = Size::SPR(n, d);
            assert_eq!(size, Size::from(u8::from(size)));
        }
    }
    /// u64 bijection: encode then decode preserves value.
    #[test]
    fn bijective_u64() {
        for &n in &OPENS {
            let size = Size::BBs(n);
            assert_eq!(size, Size::from(u64::from(size)));
        }
        for &(n, d) in &RAISES {
            let size = Size::SPR(n, d);
            assert_eq!(size, Size::from(u64::from(size)));
        }
    }
    /// into_chips: BBs variant multiplies by B_BLIND.
    #[test]
    fn into_chips_bbs() {
        let pot = 100;
        assert_eq!(Size::BBs(2).into_chips(pot), 2 * pokerkit::B_BLIND);
        assert_eq!(Size::BBs(3).into_chips(pot), 3 * pokerkit::B_BLIND);
        assert_eq!(Size::BBs(5).into_chips(pot), 5 * pokerkit::B_BLIND);
    }
    /// into_chips: SPR variant multiplies pot by fraction.
    #[test]
    fn into_chips_spr() {
        let pot = 100;
        assert_eq!(Size::SPR(1, 2).into_chips(pot), 50);
        assert_eq!(Size::SPR(1, 1).into_chips(pot), 100);
        assert_eq!(Size::SPR(2, 1).into_chips(pot), 200);
    }
    /// from_chips snaps to nearest canonical size.
    #[test]
    fn from_chips_snaps_to_nearest() {
        let pot = 100;
        let size = Size::from(Raise::new(5, pot, Street::Pref, 0));
        assert!(matches!(size, Size::BBs(2 | 3)));
        let size = Size::from(Raise::new(75, pot, Street::Flop, 0));
        assert!(matches!(size, Size::SPR(..)));
    }
    /// RAISES must contain all SPR sizes used in any raises() call.
    #[test]
    fn raise_grid_is_complete() {
        let mut all_spr = std::collections::HashSet::new();
        for street in Street::all() {
            for depth in 0..=MAX_RAISE_REPEATS {
                for size in Size::raises(street, depth) {
                    if let Size::SPR(n, d) = size {
                        all_spr.insert((n, d));
                    }
                }
            }
        }
        for pair in all_spr {
            assert!(RAISES.contains(&pair), "RAISES missing {pair:?}");
        }
    }
    /// OPENS must contain all BB values used in raises().
    #[test]
    fn opens_grid_is_complete() {
        let mut all_bbs = std::collections::HashSet::new();
        for street in Street::all() {
            for depth in 0..=MAX_RAISE_REPEATS {
                for size in Size::raises(street, depth) {
                    if let Size::BBs(n) = size {
                        all_bbs.insert(n);
                    }
                }
            }
        }
        for n in all_bbs {
            assert!(OPENS.contains(&n), "OPENS missing {n}");
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
        assert_eq!(format!("{}", Size::SPR(1, 4)), "1:4");
        assert_eq!(format!("{}", Size::SPR(3, 4)), "3:4");
    }
    /// Opening spot: `translate` with classical+nearest matches the BB-count axis.
    /// B_BLIND = 2, OPENS = [2, 3, 4, 5]. chips=7 → observed 3.5 BB → ties between BBs(3) and BBs(4); lower wins.
    #[test]
    fn translate_opening_classical_nearest_ties_low() {
        let ref mut rng = seeded();
        let out = Size::translate(Raise::new(7, 0, Street::Pref, 0), &Translation::Snap, rng);
        assert_eq!(out, Translated::Snap(Size::BBs(3)));
    }
    /// Opening spot: below the smallest open clamps to BBs(2).
    #[test]
    fn translate_opening_clamps_low() {
        let ref mut rng = seeded();
        let out = Size::translate(Raise::new(1, 0, Street::Pref, 0), &Translation::Snap, rng);
        assert_eq!(out, Translated::Snap(Size::BBs(2)));
    }
    /// Opening spot: above the largest open clamps to BBs(5).
    #[test]
    fn translate_opening_clamps_high() {
        let ref mut rng = seeded();
        let out = Size::translate(Raise::new(20, 0, Street::Pref, 0), &Translation::Snap, rng);
        assert_eq!(out, Translated::Snap(Size::BBs(5)));
    }
    /// Post-flop: classical+nearest snaps to the closest pot-fraction anchor.
    /// FLOP_0 = [1/4, 1/2, 3/4, 1:1, 2:1]. chips=60, pot=100 → 0.6 closer to 1/2 than 3/4 → SPR(1,2).
    #[test]
    fn translate_postflop_classical_nearest_snaps_to_half() {
        let ref mut rng = seeded();
        let out = Size::translate(Raise::new(60, 100, Street::Flop, 0), &Translation::Snap, rng);
        assert_eq!(out, Translated::Snap(Size::SPR(1, 2)));
    }
    /// `Harmonic` policy converges on the GS formula empirically across many samples.
    #[test]
    fn translate_harmonic_monte_carlo() {
        let ref mut rng = seeded();
        let trials = 50_000;
        let mut half_pot_hits = 0;
        for _ in 0..trials {
            match Size::translate(Raise::new(60, 100, Street::Flop, 0), &Translation::Harmonic, rng) {
                Translated::Snap(Size::SPR(1, 2)) => half_pot_hits += 1,
                Translated::Snap(Size::SPR(3, 4)) => {}
                other => panic!("unexpected: {other:?}"),
            }
        }
        let empirical = half_pot_hits as f64 / trials as f64;
        let expected = (0.75 - 0.60) * (1.0 + 0.5) / ((0.75 - 0.5) * (1.0 + 0.60));
        assert!((empirical - expected).abs() < 0.01, "empirical {empirical} vs expected {expected}");
    }
    /// String parsing roundtrip: parse(display(size)) == size.
    #[test]
    fn string_roundtrip() {
        for &n in &OPENS {
            let size = Size::BBs(n);
            let s = size.to_string();
            assert_eq!(Size::try_from(s.as_str()).unwrap(), size);
        }
        for &(n, d) in &RAISES {
            let size = Size::SPR(n, d);
            let s = size.to_string();
            assert_eq!(Size::try_from(s.as_str()).unwrap(), size);
        }
    }
}
