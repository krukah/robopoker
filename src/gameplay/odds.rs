use crate::cards::*;
use crate::mccfr::*;
use crate::*;

/// pot-normalized odds for a given raise size
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
    /// Format odds as ratio "N:N"
    pub fn ratio(&self) -> String {
        format!("{}:{}", self.0, self.1)
    }
    /// Find nearest odds value from street-appropriate grid
    /// This ensures inference uses the same grid as training
    pub fn nearest((a, b): (Chips, Chips), street: Street, depth: usize) -> Self {
        let grid = Info::raises(street, depth);
        if grid.is_empty() {
            return Self(1, 1);
        }
        let target = a as Utility / b as Utility;
        let probabilities = grid
            .iter()
            .map(|&o| Probability::from(o))
            .collect::<Vec<_>>();
        let i = probabilities
            .binary_search_by(|&p| p.partial_cmp(&target).unwrap())
            .unwrap_or_else(|i| match i {
                0 => 0,
                i if i >= probabilities.len() => probabilities.len() - 1,
                i if (target - probabilities[i - 1]).abs() < (probabilities[i] - target).abs() => {
                    i - 1
                }
                i => i,
            });
        grid[i]
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

// For +N format, odds are 1/N
// For -N format, odds are N/1
impl TryFrom<&str> for Odds {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match (s.strip_prefix('+'), s.strip_prefix('-')) {
            (Some(x), _) => Ok(Odds(1, x.parse()?)),
            (_, Some(x)) => Ok(Odds(x.parse()?, 1)),
            _ => Err(anyhow::anyhow!("odds string missing + or -")),
        }
    }
}

impl std::fmt::Display for Odds {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let p = Probability::from(*self);
        if p > 1.0 {
            write!(f, "-{}", (p * 1.0).round() as Chips)
        } else {
            write!(f, "+{}", (1.0 / p).round() as Chips)
        }
    }
}

impl Arbitrary for Odds {
    fn random() -> Self {
        use rand::prelude::IndexedRandom;
        let ref mut rng = rand::rng();
        Self::GRID.choose(rng).copied().expect("GRID is empty")
    }
}
