use rbp_core::*;

/// Pot-relative bet sizing as a fraction.
///
/// Represents raise sizes as `numerator/denominator` of the pot. For example,
/// `Odds::new(1, 2)` means a half-pot bet, `Odds::new(2, 1)` means a 2x pot overbet.
///
/// See [`Size`] for the full sizing abstraction that handles both pot-relative
/// and BB-relative interpretations.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Odds(Chips, Chips);

impl Odds {
    /// Creates new odds from numerator and denominator.
    pub const fn new(n: Chips, d: Chips) -> Self {
        Self(n, d)
    }
    /// Numerator (pot multiplier).
    pub fn numer(&self) -> Chips {
        self.0
    }
    /// Denominator (pot divisor).
    pub fn denom(&self) -> Chips {
        self.1
    }
    /// Reduces fraction to lowest terms.
    fn gcd(a: Chips, b: Chips) -> (Chips, Chips) {
        let (mut x, mut y) = (a, b);
        while y != 0 {
            (x, y) = (y, x % y);
        }
        (a / x, b / x)
    }
    /// Formats as "N:N" ratio for display.
    pub fn ratio(&self) -> String {
        format!("{}:{}", self.0, self.1)
    }
    /// Full grid for random sampling.
    pub const GRID: [Self; 10] = [
        Self(1, 4), // 0.25 pot
        Self(1, 3), // 0.33 pot
        Self(1, 2), // 0.50 pot
        Self(2, 3), // 0.66 pot
        Self(3, 4), // 0.75 pot
        Self(1, 1), // 1.00 pot
        Self(5, 4), // 1.25 pot
        Self(3, 2), // 1.50 pot
        Self(2, 1), // 2x pot
        Self(3, 1), // 3x pot
    ];
}

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

/// For +N format, odds are 1/N (pot fraction)
/// For -N format, odds are N/1 (overbet or BB multiple)
impl TryFrom<&str> for Odds {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match (s.strip_prefix('+'), s.strip_prefix('-')) {
            (Some(x), _) => Ok(Self::new(1, x.parse()?)),
            (_, Some(x)) => Ok(Self::new(x.parse()?, 1)),
            _ => Err(anyhow::anyhow!("odds string missing + or -")),
        }
    }
}

impl std::fmt::Display for Odds {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let p = Probability::from(*self);
        if p > 1.0 {
            write!(f, "-{}", p.round() as Chips)
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
