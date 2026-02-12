use super::*;
use rbp_cards::Street;
use rbp_core::*;
use std::hash::Hash;

/// An abstracted game tree transition.
///
/// Unlike [`Action`] which carries exact chip amounts, `Edge` abstracts
/// betting actions for strategy lookup across different stack depths.
///
/// # Variants
///
/// - `Draw` — Chance node (card deal)
/// - `Fold`, `Check`, `Call` — Standard player decisions
/// - `Open(Chips)` — Preflop open in BB units (e.g., 2BB, 3BB)
/// - `Raise(Odds)` — Pot-relative raise (e.g., 1/2 pot, 2x pot)
/// - `Shove` — All-in bet
#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub enum Edge {
    Draw,
    Fold,
    Check,
    Call,
    Open(Chips),
    Raise(Odds),
    Shove,
}


impl Edge {
    /// True if this is an all-in bet.
    pub fn is_shove(&self) -> bool {
        matches!(self, Edge::Shove)
    }
    /// True if this is a raise (including opens).
    pub fn is_raise(&self) -> bool {
        matches!(self, Edge::Raise(_) | Edge::Open(_))
    }
    /// True if this is a fold.
    pub fn is_folded(&self) -> bool {
        matches!(self, Edge::Fold)
    }
    /// True if this is a chance node (card deal).
    pub fn is_chance(&self) -> bool {
        matches!(self, Edge::Draw)
    }
    /// True if this is aggressive (raise, open, or shove).
    pub fn is_aggro(&self) -> bool {
        self.is_raise() || self.is_shove()
    }
    /// True if this is a player decision.
    pub fn is_choice(&self) -> bool {
        !self.is_chance()
    }
}

impl Edge {
    /// Initial regret bounds for CFR warmstart.
    ///
    /// Returns (min, max) regret to bias exploration toward certain actions.
    pub fn regret(&self) -> (Utility, Utility) {
        match self {
            Edge::Open(_) => (Utility::default(), BIAS_RAISE),
            Edge::Raise(_) => (Utility::default(), BIAS_RAISE),
            Edge::Check => (Utility::default(), BIAS_OTHER),
            Edge::Shove => (Utility::default(), BIAS_RAISE),
            Edge::Call => (Utility::default(), BIAS_OTHER),
            Edge::Fold => (Utility::default(), BIAS_FOLDS),
            Edge::Draw => panic!("chance edges have no learned regret"),
        }
    }
    /// Initial policy bounds (currently unused).
    pub fn policy(&self) -> (Probability, Probability) {
        (Probability::default(), Probability::default())
    }
}

impl From<Action> for Edge {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => Edge::Fold,
            Action::Check => Edge::Check,
            Action::Call(_) => Edge::Call,
            Action::Draw(_) => Edge::Draw,
            Action::Shove(_) => Edge::Shove,
            Action::Raise(_) => panic!("raise must be converted via Game::edgify"),
            Action::Blind(_) => panic!("blinds are not in any MCCFR trees"),
        }
    }
}

impl From<Odds> for Edge {
    fn from(odds: Odds) -> Self {
        Edge::Raise(odds)
    }
}

/// Preflop open sizes in BB units.
const OPENS_GRID: [Chips; 4] = [2, 3, 4, 8];
/// Pot-relative raise sizes as Odds (must fit in u8 10-15, Path uses 4-bit encoding).
const RAISE_GRID: [Odds; 6] = [
    Odds::new(1, 3), // 0.33 pot
    Odds::new(1, 2), // 0.50 pot
    Odds::new(2, 3), // 0.66 pot
    Odds::new(1, 1), // 1.00 pot
    Odds::new(3, 2), // 1.50 pot
    Odds::new(2, 1), // 2.00 pot
];

impl Edge {
    /// Returns available raise/open edges for a given street and depth.
    /// This is the **single source of truth** for which betting edges exist.
    pub fn raises(street: Street, depth: usize) -> Vec<Self> {
        if depth > MAX_RAISE_REPEATS {
            return vec![];
        }
        match (street, depth) {
            // Preflop depth=0: BB opens
            (Street::Pref, 0) => OPENS_GRID.iter().map(|&n| Edge::Open(n)).collect(),
            // Preflop depth>=1: pot-relative 3-bets/4-bets (1x, 1.5x, 2x pot)
            (Street::Pref, 1) => vec![
                Edge::Raise(Odds::new(1, 1)),
                Edge::Raise(Odds::new(3, 2)),
                Edge::Raise(Odds::new(2, 1)),
            ],
            (Street::Pref, _) => vec![Edge::Raise(Odds::new(1, 1)), Edge::Raise(Odds::new(2, 1))],
            // Flop (1/3 probe bet instead of 1/4 due to 4-bit Path encoding limit)
            (Street::Flop, 0) => vec![
                Edge::Raise(Odds::new(1, 3)),
                Edge::Raise(Odds::new(1, 2)),
                Edge::Raise(Odds::new(1, 1)),
                Edge::Raise(Odds::new(2, 1)),
            ],
            (Street::Flop, 1) => vec![
                Edge::Raise(Odds::new(2, 3)),
                Edge::Raise(Odds::new(1, 1)),
                Edge::Raise(Odds::new(3, 2)),
            ],
            (Street::Flop, _) => vec![Edge::Raise(Odds::new(1, 1)), Edge::Raise(Odds::new(3, 2))],
            // Turn
            (Street::Turn, 0) => vec![
                Edge::Raise(Odds::new(1, 3)),
                Edge::Raise(Odds::new(2, 3)),
                Edge::Raise(Odds::new(1, 1)),
                Edge::Raise(Odds::new(2, 1)),
            ],
            (Street::Turn, _) => vec![Edge::Raise(Odds::new(1, 1)), Edge::Raise(Odds::new(3, 2))],
            // River
            (Street::Rive, 0) => vec![
                Edge::Raise(Odds::new(1, 3)),
                Edge::Raise(Odds::new(1, 2)),
                Edge::Raise(Odds::new(1, 1)),
                Edge::Raise(Odds::new(2, 1)),
            ],
            (Street::Rive, 1) => vec![
                Edge::Raise(Odds::new(2, 3)),
                Edge::Raise(Odds::new(1, 1)),
                Edge::Raise(Odds::new(2, 1)),
            ],
            (Street::Rive, _) => vec![Edge::Raise(Odds::new(1, 1))],
        }
    }
    /// Converts edge to chip amount given pot size.
    pub fn into_chips(self, pot: Chips) -> Chips {
        match self {
            Edge::Open(n) => n * B_BLIND,
            Edge::Raise(odds) => (pot as Utility * Probability::from(odds)) as Chips,
            _ => 0,
        }
    }
}

/// u8 bijection
/// Layout: 1-5 = basic edges, 6-9 = Open(OPENS_GRID), 10-15 = Raise(RAISE_GRID)
impl From<Edge> for u8 {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::Draw => 1,
            Edge::Fold => 2,
            Edge::Check => 3,
            Edge::Call => 4,
            Edge::Shove => 5,
            Edge::Open(n) => {
                6 + OPENS_GRID
                    .iter()
                    .position(|&b| b == n)
                    .expect("invalid open size") as u8
            }
            Edge::Raise(odds) => {
                10 + RAISE_GRID
                    .iter()
                    .position(|&o| o == odds)
                    .expect("invalid raise odds") as u8
            }
        }
    }
}
impl From<u8> for Edge {
    fn from(value: u8) -> Self {
        match value {
            1 => Edge::Draw,
            2 => Edge::Fold,
            3 => Edge::Check,
            4 => Edge::Call,
            5 => Edge::Shove,
            6..=9 => Edge::Open(OPENS_GRID[value as usize - 6]),
            10..=15 => Edge::Raise(RAISE_GRID[value as usize - 10]),
            _ => unreachable!("invalid edge encoding: {}", value),
        }
    }
}

/// u64 bijection with backwards compatibility for old Size encoding.
/// Old format: tag 4 with bit 19 set = BBs → decoded as Open
/// New format: tag 6 = Open, tag 4 = Raise(Odds)
impl From<u64> for Edge {
    fn from(value: u64) -> Self {
        match value & 0b111 {
            0 => Self::Draw,
            1 => Self::Fold,
            2 => Self::Check,
            3 => Self::Call,
            4 => {
                // Check for old BBs encoding (bit 19 set)
                if value & (1 << 19) != 0 {
                    // Old format: Raise(BBs(n)) → convert to Open(n)
                    Self::Open(((value >> 3) & 0xFF) as Chips)
                } else {
                    // SPR format: Raise(Odds(n, d))
                    Self::Raise(Odds::new(
                        ((value >> 3) & 0xFF) as Chips,
                        ((value >> 11) & 0xFF) as Chips,
                    ))
                }
            }
            5 => Self::Shove,
            6 => Self::Open(((value >> 3) & 0xFF) as Chips),
            _ => unreachable!("invalid edge encoding"),
        }
    }
}
impl From<Edge> for u64 {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::Draw => 0,
            Edge::Fold => 1,
            Edge::Check => 2,
            Edge::Call => 3,
            Edge::Raise(odds) => 4 | ((odds.numer() as u64) << 3) | ((odds.denom() as u64) << 11),
            Edge::Shove => 5,
            Edge::Open(n) => 6 | ((n as u64) << 3),
        }
    }
}

impl TryFrom<&str> for Edge {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "?" => Ok(Edge::Draw),
            "F" => Ok(Edge::Fold),
            "*" => Ok(Edge::Call),
            "O" => Ok(Edge::Check),
            "!" => Ok(Edge::Shove),
            s if s.ends_with("bb") => {
                let n = s
                    .strip_suffix("bb")
                    .and_then(|x| x.parse::<Chips>().ok())
                    .ok_or_else(|| anyhow::anyhow!("invalid bb format"))?;
                Ok(Edge::Open(n))
            }
            s if s.contains(':') => {
                let (n, d) = s
                    .split_once(':')
                    .ok_or_else(|| anyhow::anyhow!("invalid ratio format"))?;
                let n = n.parse::<Chips>()?;
                let d = d.parse::<Chips>()?;
                Ok(Edge::Raise(Odds::new(n, d)))
            }
            _ => Err(anyhow::anyhow!("invalid edge format: {}", s)),
        }
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edge::Draw => write!(f, "?"),
            Edge::Fold => write!(f, "F"),
            Edge::Call => write!(f, "*"),
            Edge::Check => write!(f, "O"),
            Edge::Shove => write!(f, "!"),
            Edge::Open(n) => write!(f, "{}bb", n),
            Edge::Raise(odds) => write!(f, "{}:{}", odds.numer(), odds.denom()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_cards::Street;
    #[test]
    fn bijective_u8() {
        let edges = vec![Edge::Draw, Edge::Fold, Edge::Check, Edge::Call, Edge::Shove];
        let opens = OPENS_GRID.iter().map(|&n| Edge::Open(n));
        let raises = RAISE_GRID.iter().map(|&o| Edge::Raise(o));
        for edge in edges.into_iter().chain(opens).chain(raises) {
            assert_eq!(
                edge,
                Edge::from(u8::from(edge)),
                "u8 roundtrip failed for {:?}",
                edge
            );
        }
    }
    #[test]
    fn bijective_u64() {
        let edges = vec![Edge::Draw, Edge::Fold, Edge::Check, Edge::Call, Edge::Shove];
        let opens = OPENS_GRID.iter().map(|&n| Edge::Open(n));
        let raises = RAISE_GRID.iter().map(|&o| Edge::Raise(o));
        for edge in edges.into_iter().chain(opens).chain(raises) {
            assert_eq!(
                edge,
                Edge::from(u64::from(edge)),
                "u64 roundtrip failed for {:?}",
                edge
            );
        }
    }
    #[test]
    fn string_roundtrip() {
        let edges = vec![
            Edge::Draw,
            Edge::Fold,
            Edge::Check,
            Edge::Call,
            Edge::Shove,
            Edge::Open(2),
            Edge::Open(3),
            Edge::Open(8),
            Edge::Raise(Odds::new(1, 2)),
            Edge::Raise(Odds::new(1, 1)),
            Edge::Raise(Odds::new(3, 2)),
            Edge::Raise(Odds::new(2, 1)),
        ];
        for edge in edges {
            let s = edge.to_string();
            let parsed = Edge::try_from(s.as_str()).unwrap();
            assert_eq!(edge, parsed, "string roundtrip failed for {:?}", edge);
        }
    }
    #[test]
    fn backwards_compat_u64_bbs() {
        // Old encoding: Edge::Raise(Size::BBs(8)) = 4 | (1 << 19) | (8 << 3)
        let old_bbs_8 = 4u64 | (1 << 19) | (8 << 3);
        assert_eq!(Edge::from(old_bbs_8), Edge::Open(8));
        let old_bbs_2 = 4u64 | (1 << 19) | (2 << 3);
        assert_eq!(Edge::from(old_bbs_2), Edge::Open(2));
    }
    #[test]
    fn raises_preflop_depth0_returns_opens() {
        let edges = Edge::raises(Street::Pref, 0);
        assert!(edges.iter().all(|e| matches!(e, Edge::Open(_))));
        assert_eq!(edges.len(), 4);
    }
    #[test]
    fn raises_postflop_returns_raises() {
        for street in [Street::Flop, Street::Turn, Street::Rive] {
            for depth in 0..=2 {
                let edges = Edge::raises(street, depth);
                assert!(edges.iter().all(|e| matches!(e, Edge::Raise(_))));
            }
        }
    }
}

impl Arbitrary for Edge {
    fn random() -> Self {
        use rand::prelude::IndexedRandom;
        match rand::random_range(0..7) {
            0 => Self::Draw,
            1 => Self::Fold,
            2 => Self::Check,
            3 => Self::Call,
            4 => Self::Shove,
            5 => Self::Open(*OPENS_GRID.choose(&mut rand::rng()).unwrap()),
            6 => Self::Raise(*RAISE_GRID.choose(&mut rand::rng()).unwrap()),
            _ => unreachable!(),
        }
    }
}

