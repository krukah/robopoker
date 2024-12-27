use crate::Arbitrary;
use crate::Chips;
use crate::Probability;
use crate::Utility;

/// pot odds for a given raise size, relative to the pot
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Odds(pub Chips, pub Chips);

impl From<Odds> for Probability {
    fn from(odds: Odds) -> Self {
        odds.0 as Probability / odds.1 as Probability
    }
}

impl From<(Chips, Chips)> for Odds {
    fn from((a, b): (Chips, Chips)) -> Self {
        let (a, b) = Self::gcd(a, b);
        Self(a, b)
    }
}

impl Odds {
    fn gcd(a: Chips, b: Chips) -> (Chips, Chips) {
        let (mut a, mut b) = (a, b);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        (a, b)
    }
    pub fn nearest((a, b): (Chips, Chips)) -> Self {
        let odds = a as Utility / b as Utility;
        Odds::GRID[Odds::GRID
            .map(|o| Probability::from(o)) // pre-sorted
            .binary_search_by(|p| p.partial_cmp(&odds).expect("not NaN"))
            .unwrap_or_else(|i| i.saturating_sub(1))]
    }
    pub const GRID: [Self; 10] = Self::PREF_RAISES;
    pub const PREF_RAISES: [Self; 10] = [
        Self(1, 4), // 0.25
        Self(1, 3), // 0.33
        Self(1, 2), // 0.50
        Self(2, 3), // 0.66
        Self(3, 4), // 0.75
        Self(1, 1), // 1.00
        Self(3, 2), // 1.50
        Self(2, 1), // 2.00
        Self(3, 1), // 3.00
        Self(4, 1), // 4.00
    ];
    pub const FLOP_RAISES: [Self; 5] = [
        Self(1, 2), // 0.50
        Self(3, 4), // 0.75
        Self(1, 1), // 1.00
        Self(3, 2), // 1.50
        Self(2, 1), // 2.00
    ];
    pub const LATE_RAISES: [Self; 2] = [
        Self(1, 2), // 0.50
        Self(1, 1), // 1.00
    ];
    pub const LAST_RAISES: [Self; 1] = [
        Self(1, 1), // 1.00
    ];
}

impl std::fmt::Display for Odds {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl Arbitrary for Odds {
    fn random() -> Self {
        use rand::seq::SliceRandom;
        Self::GRID
            .choose(&mut rand::thread_rng())
            .copied()
            .expect("GRID is empty")
    }
}
